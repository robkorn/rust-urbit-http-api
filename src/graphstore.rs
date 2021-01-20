use crate::helper::get_current_da_time;
use crate::{Channel, Result, UrbitAPIError};
use json::{object, JsonValue};
use std::time::{SystemTime, UNIX_EPOCH};

/// A struct which exposes Graph Store functionality
pub struct GraphStore<'a> {
    pub channel: &'a mut Channel,
}

impl<'a> GraphStore<'a> {
    /// Issue a `post` to Graph Store
    pub fn post(
        &mut self,
        resource_ship: &str,
        resource_name: &str,
        contents: Vec<JsonValue>,
    ) -> Result<()> {
        // Add the ~ to the ship name to be used within the post as author
        let ship = format!("~{}", self.channel.ship_interface.ship_name);

        // Get the current Unix Time
        let unix_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // The index. For chat this is the `now` in Urbit as an atom.
        // Need to implement properly with `now` still.
        let index = format!("/{}", get_current_da_time());

        // Creating the json by adding the index dynamically for the key
        // for the inner part of the json
        let mut nodes_json = object!();
        nodes_json[index.clone()] = object! {
                        "post": {
                            "author": ship.clone(),
                            "index": index.clone(),
                            "time-sent": unix_time,
                            "contents": contents,
                            "hash": null,
                            "signatures": []
                        },
                        "children": null
        };
        // Rest of the json creation
        let poke_json = object! {
            "add-nodes": {
                "resource": {
                    "ship": resource_ship,
                    "name": resource_name
                },
            "nodes": nodes_json
            }
        };

        let resp = (&mut self.channel).poke("graph-push-hook", "graph-update", &poke_json)?;

        if resp.status().as_u16() == 204 {
            Ok(())
        } else {
            return Err(UrbitAPIError::FailedToIssueGraphStorePost(
                resource_name.to_string(),
            ));
        }
    }
}
