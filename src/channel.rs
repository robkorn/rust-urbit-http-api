use crate::error::{Result, UrbitAPIError};
use crate::interface::ShipInterface;
use crate::subscription::{CreationID, Subscription};
use eventsource_threaded::{EventSource, ReceiverSource};
use json::object;
use rand::Rng;
use reqwest::blocking::Response;
use reqwest::header::HeaderMap;
use reqwest::Url;
use std::time::SystemTime;

// A Channel which is used to interact with a ship
#[derive(Debug)]
pub struct Channel {
    /// `ShipInterface` this channel is created from
    pub ship_interface: ShipInterface,
    /// The uid of the channel
    pub uid: String,
    /// The url of the channel
    pub url: String,
    // The list of `Subscription`s for this channel
    pub subscription_list: Vec<Subscription>,
    // / The `EventSource` for this channel which reads all of
    // / the SSE events.
    event_receiver: ReceiverSource,
    /// The current number of messages that have been sent out (which are
    /// also defined as message ids) via this `Channel`
    pub message_id_count: u64,
}

impl Channel {
    /// Create a new channel
    pub fn new(ship_interface: ShipInterface) -> Result<Channel> {
        let mut rng = rand::thread_rng();
        // Defining the uid as UNIX time, or random if error
        let uid = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => rng.gen(),
        }
        .to_string();

        // Channel url
        let channel_url = format!("{}/~/channel/{}", &ship_interface.url, uid);
        // Opening channel request json
        let mut body = json::parse(r#"[]"#).unwrap();
        body[0] = object! {
                "id": 1,
                "action": "poke",
                "ship": ship_interface.ship_name.clone(),
                "app": "hood",
                "mark": "helm-hi",
                "json": "Opening channel",
        };

        // Make the put request to create the channel.
        let resp = ship_interface.send_put_request(&channel_url, &body)?;

        if resp.status().as_u16() == 204 {
            // Create cookie header with the ship session auth val
            let mut headers = HeaderMap::new();
            headers.append("cookie", ship_interface.session_auth.clone());
            // Create the receiver
            let receiver = EventSource::new(Url::parse(&channel_url).unwrap(), headers);

            return Ok(Channel {
                ship_interface: ship_interface,
                uid: uid,
                url: channel_url,
                subscription_list: vec![],
                event_receiver: receiver,
                message_id_count: 2,
            });
        } else {
            return Err(UrbitAPIError::FailedToCreateNewChannel);
        }
    }

    /// Acquires and returns the current `message_id_count` from the
    /// `ShipInterface` that this channel was created from while also
    /// increase said value by 1.
    pub fn get_and_raise_message_id_count(&mut self) -> u64 {
        let current_id_count = self.message_id_count;
        self.message_id_count += 1;
        current_id_count
    }

    /// Sends a poke over the channel
    pub fn poke(&mut self, app: &str, mark: &str, json: &str) -> Result<Response> {
        let mut body = json::parse(r#"[]"#).unwrap();
        body[0] = object! {
                "id": self.get_and_raise_message_id_count(),
                "action": "poke",
                "ship": self.ship_interface.ship_name.clone(),
                "app": app,
                "mark": mark,
                "json": json,
        };

        // Make the put request for the poke
        self.ship_interface.send_put_request(&self.url, &body)
    }

    /// Create a new `Subscription` and thus subscribes to events on the
    /// ship with the provided app/path.
    pub fn create_new_subscription(&mut self, app: &str, path: &str) -> Result<CreationID> {
        // Saves the message id to be reused
        let creation_id = self.get_and_raise_message_id_count();
        // Create the json body
        let mut body = json::parse(r#"[]"#).unwrap();
        body[0] = object! {
                "id": creation_id,
                "action": "subscribe",
                "ship": self.ship_interface.ship_name.clone(),
                "app": app.to_string(),
                "path": path.to_string(),
        };

        // Make the put request to create the channel.
        let resp = self.ship_interface.send_put_request(&self.url, &body)?;

        if resp.status().as_u16() == 204 {
            // Create the `Subscription`
            let sub = Subscription {
                channel_uid: self.uid.clone(),
                creation_id: creation_id,
                app: app.to_string(),
                path: path.to_string(),
                message_list: vec![],
            };
            // Add the `Subscription` to the list
            self.subscription_list.push(sub.clone());
            return Ok(creation_id);
        } else {
            return Err(UrbitAPIError::FailedToCreateNewSubscription);
        }
    }

    /// Parses SSE messages for this channel and moves them into
    /// the proper corresponding `Subscription`'s `message_list`.
    pub fn parse_event_messages(&mut self) {
        let rec = &mut self.event_receiver;

        // Consume all messages
        loop {
            if let Ok(event_res) = rec.try_recv() {
                if let Err(e) = &event_res {
                    println!("Error Event: {}", e);
                }
                if let Ok(event) = event_res {
                    // Go through all subscriptions and find which
                    // subscription this event is for.
                    for sub in &mut self.subscription_list {
                        // If adding the message succeeded (because found
                        // correct `Subscription`) then stop.
                        if let Some(_) = sub.add_to_message_list(&event) {
                            // Send an ack for the processed event
                            // Using unwrap because `add_to_message_list`
                            // already does error checking.
                            let eid: u64 = event.id.unwrap().parse().unwrap();
                            let mut json = json::parse(r#"[]"#).unwrap();
                            json[0] = object! {
                                "id": self.message_id_count,
                                "action": "ack",
                                "event-id": eid,
                            };
                            self.message_id_count += 1;
                            let ack_res = self.ship_interface.send_put_request(&self.url, &json);
                            break;
                        }
                    }
                }
                continue;
            }
            break;
        }
    }

    /// Finds the first `Subscription` in the list which has a matching
    /// `app` and `path`;
    pub fn find_subscription(&mut self, app: &str, path: &str) -> Option<&mut Subscription> {
        for sub in &mut self.subscription_list {
            if sub.app == app && sub.path == path {
                return Some(sub);
            }
        }
        None
    }

    /// Finds the first `Subscription` in the list which has a matching
    /// `app` and `path`, removes it from the list, and tells the ship
    /// that you are unsubscribing. Returns `None` if failed to find
    /// a subscription with a matching app & path.
    pub fn unsubscribe(&mut self, app: &str, path: &str) -> Option<bool> {
        let index = self
            .subscription_list
            .iter()
            .position(|s| s.app == app && s.path == path)?;
        self.subscription_list.remove(index);
        Some(true)
    }

    /// Deletes the channel
    pub fn delete_channel(self) {
        let mut json = json::parse(r#"[]"#).unwrap();
        json[0] = object! {
            "id": self.message_id_count,
            "action": "delete",
        };
        let res = self.ship_interface.send_put_request(&self.url, &json);
        std::mem::drop(self);
    }
}
