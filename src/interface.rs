use crate::channel::Channel;
use crate::error::{Result, UrbitAPIError};
use json::JsonValue;
use reqwest::blocking::{Client, Response};
use reqwest::header::{HeaderValue, COOKIE};

// The struct which holds the details for connecting to a given Urbit ship
#[derive(Debug, Clone)]
pub struct ShipInterface {
    /// The URL of the ship given as `http://ip:port` such as
    /// `http://0.0.0.0:8080`.
    pub url: String,
    /// The session auth string header value
    pub session_auth: HeaderValue,
    /// The ship name
    pub ship_name: String,
    /// The Reqwest `Client` to be reused for making requests
    req_client: Client,
}

impl ShipInterface {
    /// Logs into the given ship and creates a new `ShipInterface`.
    /// `ship_url` should be `http://ip:port` of the given ship. Example:
    /// `http://0.0.0.0:8080`. `ship_code` is the code acquire from your ship
    /// by typing `+code` in dojo.
    pub fn new(ship_url: &str, ship_code: &str) -> Result<ShipInterface> {
        let client = Client::new();
        let login_url = format!("{}/~/login", ship_url);
        let resp = client
            .post(&login_url)
            .body("password=".to_string() + &ship_code)
            .send()?;

        // Check for status code
        if resp.status().as_u16() != 204 {
            return Err(UrbitAPIError::FailedToLogin);
        }

        // Acquire the session auth header value
        let session_auth = resp
            .headers()
            .get("set-cookie")
            .ok_or(UrbitAPIError::FailedToLogin)?;

        // Convert sessions auth to a string
        let auth_string = session_auth
            .to_str()
            .map_err(|_| UrbitAPIError::FailedToLogin)?;

        // Trim the auth string to acquire the ship name
        let ship_name = &auth_string[9..auth_string.find('=').unwrap()];

        Ok(ShipInterface {
            url: ship_url.to_string(),
            session_auth: session_auth.clone(),
            ship_name: ship_name.to_string(),
            req_client: client,
        })
    }

    /// Create a `Channel` using this `ShipInterface`
    pub fn create_channel(&self) -> Result<Channel> {
        Channel::new(self.clone())
    }

    // Send a put request using the `ShipInterface`
    pub fn send_put_request(&self, url: &str, body: &JsonValue) -> Result<Response> {
        let json = body.dump();
        let resp = self
            .req_client
            .put(url)
            .header(COOKIE, self.session_auth.clone())
            .header("Content-Type", "application/json")
            .body(json);

        Ok(resp.send()?)
    }

    /// Sends a scry to the ship
    pub fn scry(&self, app: &str, path: &str, mark: &str) -> Result<Response> {
        let scry_url = format!("{}/~/scry/{}{}.{}", self.url, app, path, mark);
        let resp = self
            .req_client
            .get(&scry_url)
            .header(COOKIE, self.session_auth.clone())
            .header("Content-Type", "application/json");

        Ok(resp.send()?)
    }

    /// Run a thread via spider
    pub fn spider(
        &self,
        input_mark: &str,
        output_mark: &str,
        thread_name: &str,
        body: &JsonValue,
    ) -> Result<Response> {
        let json = body.dump();
        let spider_url = format!(
            "{}/spider/{}/{}/{}.json",
            self.url, input_mark, thread_name, output_mark
        );

        let resp = self
            .req_client
            .post(&spider_url)
            .header(COOKIE, self.session_auth.clone())
            .header("Content-Type", "application/json")
            .body(json);

        Ok(resp.send()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subscription::Subscription;
    use json::object;
    #[test]
    // Verify that we can login to a local `~zod` dev ship.
    fn can_login() {
        let ship_interface =
            ShipInterface::new("http://0.0.0.0:8080", "lidlut-tabwed-pillex-ridrup").unwrap();
    }

    #[test]
    // Verify that we can create a channel
    fn can_create_channel() {
        let ship_interface =
            ShipInterface::new("http://0.0.0.0:8080", "lidlut-tabwed-pillex-ridrup").unwrap();
        let channel = ship_interface.create_channel().unwrap();
        channel.delete_channel();
    }

    #[test]
    // Verify that we can create a channel
    fn can_subscribe() {
        let ship_interface =
            ShipInterface::new("http://0.0.0.0:8080", "lidlut-tabwed-pillex-ridrup").unwrap();
        let mut channel = ship_interface.create_channel().unwrap();
        channel
            .create_new_subscription("chat-view", "/primary")
            .unwrap();

        channel.find_subscription("chat-view", "/primary");
        channel.unsubscribe("chat-view", "/primary");
        channel.delete_channel();
    }

    #[test]
    // Verify that we can make a poke
    fn can_poke() {
        let ship_interface =
            ShipInterface::new("http://0.0.0.0:8080", "lidlut-tabwed-pillex-ridrup").unwrap();
        let mut channel = ship_interface.create_channel().unwrap();
        let poke_res = channel
            .poke("hood", "helm-hi", &"A poke has been made".into())
            .unwrap();
        assert!(poke_res.status().as_u16() == 204);
        channel.delete_channel();
    }

    #[test]
    // Verify we can scry
    fn can_scry() {
        let mut ship_interface =
            ShipInterface::new("http://0.0.0.0:8080", "lidlut-tabwed-pillex-ridrup").unwrap();
        let scry_res = ship_interface.scry("graph-store", "/keys", "json").unwrap();

        assert!(scry_res.status().as_u16() == 200);
    }

    #[test]
    // Verify we can run threads
    fn can_spider() {
        let mut ship_interface =
            ShipInterface::new("http://0.0.0.0:8080", "lidlut-tabwed-pillex-ridrup").unwrap();
        let create_req = object! {
            "create": {
                "resource": {
                    "ship": "~zod",
                    "name": "test",
                },
                "title": "Testing creation",
                "description": "test",
                "associated": {
                    "policy": {
                        "invite": {
                            "pending": []
                        }
                    }
                },
                "module": "chat",
                "mark": "graph-validator-chat"
            }
        };

        let spider_res = ship_interface
            .spider("graph-view-action", "json", "graph-create", &create_req)
            .unwrap();

        assert!(spider_res.status().as_u16() == 200);
    }
}
