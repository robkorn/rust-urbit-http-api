use crate::helper::get_current_da_time;
use crate::{Channel, Result, UrbitAPIError};
use json::{object, JsonValue};
use std::time::{SystemTime, UNIX_EPOCH};

/// A struct which exposes Graph Store functionality
pub struct GraphStore<'a> {
    pub channel: &'a mut Channel,
}

impl<'a> GraphStore<'a> {
    /// Issue a `post` to Graph Store.
    /// On success returns the index of the newly added node.
    pub fn post(
        &mut self,
        resource_ship: &str,
        resource_name: &str,
        contents: Vec<JsonValue>,
    ) -> Result<String> {
        // The index. For chat the default is current time in `@da` encoding with a `/` in front.
        let index = format!("/{}", get_current_da_time());

        self.post_custom_index(resource_ship, resource_name, contents, &index)
    }

    /// Issue a `post` to Graph Store using a custom index.
    /// On success returns the index of the newly added node.
    pub fn post_custom_index(
        &mut self,
        resource_ship: &str,
        resource_name: &str,
        contents: Vec<JsonValue>,
        index: &str,
    ) -> Result<String> {
        // Add the ~ to the ship name to be used within the post as author
        let ship = format!("~{}", self.channel.ship_interface.ship_name);

        // Get the current Unix Time
        let unix_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

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

        // Using `?` to ensure adding the node was a success, else return error.
        self.add_nodes(resource_ship, resource_name, nodes_json)?;
        Ok(index.to_string())
    }

    /// Add nodes to Graph Store
    pub fn add_nodes(
        &mut self,
        resource_ship: &str,
        resource_name: &str,
        nodes_json: JsonValue,
    ) -> Result<()> {
        let prepped_json = object! {
            "add-nodes": {
                "resource": {
                    "ship": resource_ship,
                    "name": resource_name
                },
            "nodes": nodes_json
            }
        };

        let resp = (&mut self.channel).poke("graph-push-hook", "graph-update", &prepped_json)?;

        if resp.status().as_u16() == 204 {
            Ok(())
        } else {
            return Err(UrbitAPIError::FailedToAddNodesToGraphStore(
                resource_name.to_string(),
            ));
        }
    }

    /// Remove nodes from Graph Store using the provided list of indices
    pub fn remove_nodes(
        &mut self,
        resource_ship: &str,
        resource_name: &str,
        indices: Vec<&str>,
    ) -> Result<()> {
        let prepped_json = object! {
            "remove-nodes": {
                "resource": {
                    "ship": resource_ship,
                    "name": resource_name
                },
            "indices": indices
            }
        };

        let resp = (&mut self.channel).poke("graph-push-hook", "graph-update", &prepped_json)?;

        if resp.status().as_u16() == 204 {
            Ok(())
        } else {
            return Err(UrbitAPIError::FailedToRemoveNodesFromGraphStore(
                resource_name.to_string(),
            ));
        }
    }

    /// Acquire a graph from Graph Store
    pub fn get_graph(&mut self, resource_ship: &str, resource_name: &str) -> Result<String> {
        let path = format!("/graph/{}/{}", resource_ship, resource_name);
        let res = self
            .channel
            .ship_interface
            .scry("graph-store", &path, "json")?;

        if res.status().as_u16() == 204 {
            if let Ok(body) = res.text() {
                return Ok(body);
            }
        }
        return Err(UrbitAPIError::FailedToGetGraph(resource_name.to_string()));
    }

    /// Archive a graph in Graph Store
    pub fn archive_graph(&mut self, resource_ship: &str, resource_name: &str) -> Result<String> {
        let path = format!("/archive/{}/{}", resource_ship, resource_name);
        let res = self
            .channel
            .ship_interface
            .scry("graph-store", &path, "json")?;

        if res.status().as_u16() == 204 {
            if let Ok(body) = res.text() {
                return Ok(body);
            }
        }
        return Err(UrbitAPIError::FailedToArchiveGraph(
            resource_name.to_string(),
        ));
    }

    /// Remove graph from Graph Store
    pub fn remove_graph(&mut self, resource_ship: &str, resource_name: &str) -> Result<()> {
        let prepped_json = object! {
            "remove-graph": {
                "resource": {
                    "ship": resource_ship,
                    "name": resource_name
                }
            }
        };

        let resp = (&mut self.channel).poke("graph-push-hook", "graph-update", &prepped_json)?;

        if resp.status().as_u16() == 204 {
            Ok(())
        } else {
            return Err(UrbitAPIError::FailedToRemoveGraphFromGraphStore(
                resource_name.to_string(),
            ));
        }
    }
}
