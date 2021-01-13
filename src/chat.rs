use crate::{Channel, Result, UrbitAPIError};
use json::object;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Chat<'a> {
    pub channel: &'a mut Channel,
}

impl<'a> Chat<'a> {
    pub fn send_message(&mut self, chat_name: &str, message: &str) -> Result<()> {
        // Add the ~ to the ship name to be used within the poke json
        let ship = format!("~{}", self.channel.ship_interface.ship_name);

        // Get the current Unix Time
        let unix_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // The index. For chat this is the `now` in Urbit as an atom.
        // Need to implement properly with `now` still.
        let index = format!("/{}", unix_time);

        // Creating the json by adding the index dynamically for the key
        // for the inner part of the json
        let mut nodes_json = object!();
        nodes_json[index.clone()] = object! {
                        "post": {
                            "author": ship.clone(),
                            "index": index.clone(),
                            "time-sent": unix_time,
                            "contents": [{
                                "text": message
                            }],
                            "hash": null,
                            "signatures": []
                        },
                        "children": null
        };
        // Rest of the json creation
        let poke_json = object! {
            "add-nodes": {
                "resource": {
                    "ship": ship.clone(),
                    "name": chat_name
                },
            "nodes": nodes_json
            }
        };

        let resp = (&mut self.channel).poke("graph-push-hook", "graph-update", poke_json)?;

        if resp.status().as_u16() == 204 {
            Ok(())
        } else {
            return Err(UrbitAPIError::FailedToSendChatMessage(
                chat_name.to_string(),
            ));
        }
    }
}
