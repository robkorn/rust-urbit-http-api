use crate::graph::{Graph, Node, NodeContents};
use crate::helper::{get_current_da_time, get_current_time, index_dec_to_ud};
use crate::{Channel, Result, UrbitAPIError};
use json::{object, JsonValue};

/// The type of module a given graph is.
pub enum Module {
    Chat,
    Notebook,
    Collection,
    Null,
}

/// A struct which exposes Graph Store functionality
pub struct GraphStore<'a> {
    pub channel: &'a mut Channel,
}

impl<'a> GraphStore<'a> {
    /// Create a new Graph Store node using defaults from the connected ship and local time.
    /// This is a wrapper method around `Node::new()` which fills out a lot of boilerplate.
    pub fn new_node(&self, contents: &NodeContents) -> Node {
        // Add the ~ to the ship name to be used within the post as author
        let ship = format!("~{}", self.channel.ship_interface.ship_name);
        // The index. For chat the default is current `@da` time as atom encoding with a `/` in front.
        let index = format!("/{}", get_current_da_time());

        // Get the current Unix Time
        let unix_time = get_current_time();

        Node::new(
            index,
            ship.clone(),
            unix_time,
            vec![],
            contents.clone(),
            None,
        )
    }

    /// Create a new Graph Store node using a specified index and creation time
    /// using the connected ship as author
    pub fn new_node_specified(
        &self,
        node_index: &str,
        unix_time: u64,
        contents: &NodeContents,
    ) -> Node {
        // Add the ~ to the ship name to be used within the post as author
        let ship = format!("~{}", self.channel.ship_interface.ship_name);
        Node::new(
            node_index.to_string(),
            ship.clone(),
            unix_time,
            vec![],
            contents.clone(),
            None,
        )
    }

    /// Add node to Graph Store
    pub fn add_node(
        &mut self,
        resource_ship: &str,
        resource_name: &str,
        node: &Node,
    ) -> Result<()> {
        let prepped_json = object! {
            "add-nodes": {
                "resource": {
                    "ship": resource_ship,
                    "name": resource_name
                },
            "nodes": node.to_json()
            }
        };

        let resp = (&mut self.channel).poke("graph-push-hook", "graph-update-2", &prepped_json)?;

        if resp.status().as_u16() == 204 {
            Ok(())
        } else {
            return Err(UrbitAPIError::FailedToAddNodesToGraphStore(
                resource_name.to_string(),
            ));
        }
    }

    /// Add node to Graph Store via spider thread
    pub fn add_node_spider(
        &mut self,
        resource_ship: &str,
        resource_name: &str,
        node: &Node,
    ) -> Result<()> {
        let prepped_json = object! {
            "add-nodes": {
                "resource": {
                    "ship": resource_ship,
                    "name": resource_name
                },
            "nodes": node.to_json()
            }
        };

        let resp = self.channel.ship_interface.spider(
            "graph-update",
            "graph-view-action",
            "graph-add-nodes",
            &prepped_json,
        )?;

        if resp.status().as_u16() == 200 {
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

        let resp = (&mut self.channel).poke("graph-push-hook", "graph-update-2", &prepped_json)?;

        if resp.status().as_u16() == 204 {
            Ok(())
        } else {
            return Err(UrbitAPIError::FailedToRemoveNodesFromGraphStore(
                resource_name.to_string(),
            ));
        }
    }

    /// Acquire a node from Graph Store
    pub fn get_node(
        &mut self,
        resource_ship: &str,
        resource_name: &str,
        node_index: &str,
    ) -> Result<Node> {
        let path_nodes = index_dec_to_ud(node_index);
        let path = format!("/node/{}/{}{}", resource_ship, resource_name, &path_nodes);
        let res = self
            .channel
            .ship_interface
            .scry("graph-store", &path, "json")?;

        // If successfully acquired node json
        if res.status().as_u16() == 200 {
            if let Ok(body) = res.text() {
                if let Ok(node_json) = json::parse(&body) {
                    return Node::from_graph_update_json(&node_json);
                }
            }
        }
        // Else return error
        Err(UrbitAPIError::FailedToGetGraphNode(format!(
            "/{}/{}/{}",
            resource_ship, resource_name, node_index
        )))
    }

    /// Acquire a subset of children of a node from Graph Store by specifying the start and end indices
    /// of the subset children.
    pub fn get_node_subset(
        &mut self,
        resource_ship: &str,
        resource_name: &str,
        node_index: &str,
        start_index: &str,
        end_index: &str,
    ) -> Result<Graph> {
        let path = format!(
            "/node-children-subset/{}/{}/{}/{}/{}",
            resource_ship, resource_name, node_index, end_index, start_index
        );
        let res = self
            .channel
            .ship_interface
            .scry("graph-store", &path, "json")?;

        // If successfully acquired node json
        if res.status().as_u16() == 200 {
            if let Ok(body) = res.text() {
                if let Ok(graph_json) = json::parse(&body) {
                    return Graph::from_json(graph_json);
                }
            }
        }
        // Else return error
        Err(UrbitAPIError::FailedToGetGraph(resource_name.to_string()))
    }

    /// Create a new graph on the connected Urbit ship that is managed
    /// (meaning associated with a specific group)
    pub fn create_managed_graph(
        &mut self,
        graph_resource_name: &str,
        graph_title: &str,
        graph_description: &str,
        graph_module: Module,
        managed_group_ship: &str,
        managed_group_name: &str,
    ) -> Result<()> {
        let create_req = object! {
            "create": {
                "resource": {
                    "ship": format!("~{}", &self.channel.ship_interface.ship_name),
                    "name": graph_resource_name
                },
                "title": graph_title,
                "description": graph_description,
                "associated": {
                    "group": {
                        "ship": managed_group_ship,
                        "name": managed_group_name,
                    },
                },
                "module": module_to_validator_string(&graph_module),
                "mark": module_to_mark(&graph_module)
            }
        };

        let resp = self
            .channel
            .ship_interface
            .spider("graph-view-action", "json", "graph-create", &create_req)
            .unwrap();

        if resp.status().as_u16() == 200 {
            Ok(())
        } else {
            Err(UrbitAPIError::FailedToCreateGraphInShip(
                graph_resource_name.to_string(),
            ))
        }
    }

    /// Create a new graph on the connected Urbit ship that is unmanaged
    /// (meaning not associated with any group)
    pub fn create_unmanaged_graph(
        &mut self,
        graph_resource_name: &str,
        graph_title: &str,
        graph_description: &str,
        graph_module: Module,
    ) -> Result<()> {
        let create_req = object! {
            "create": {
                "resource": {
                    "ship": self.channel.ship_interface.ship_name_with_sig(),
                    "name": graph_resource_name
                },
                "title": graph_title,
                "description": graph_description,
                "associated": {
                    "policy": {
                        "invite": {
                            "pending": []
                        }
                    }
                },
                "module": module_to_validator_string(&graph_module),
                "mark": module_to_mark(&graph_module)
            }
        };

        let resp = self
            .channel
            .ship_interface
            .spider("graph-view-action", "json", "graph-create", &create_req)
            .unwrap();

        if resp.status().as_u16() == 200 {
            Ok(())
        } else {
            Err(UrbitAPIError::FailedToCreateGraphInShip(
                graph_resource_name.to_string(),
            ))
        }
    }

    // /// Create a new graph on the connected Urbit ship that is unmanaged
    // /// (meaning not associated with any group) and "raw", meaning created
    // /// directly via poking graph-store and not set up to deal with networking
    // pub fn create_unmanaged_graph_raw(&mut self, graph_resource_name: &str) -> Result<()> {
    //     // [%add-graph =resource =graph mark=(unit mark)]

    //     let prepped_json = object! {
    //         "add-graph": {
    //             "resource": {
    //                 "ship": self.channel.ship_interface.ship_name_with_sig(),
    //                 "name": graph_resource_name
    //             },
    //         "graph": "",
    //         "mark": "",

    //         }
    //     };

    //     let resp = (&mut self.channel).poke("graph-store", "graph-update-2", &prepped_json)?;

    //     if resp.status().as_u16() == 200 {
    //         Ok(())
    //     } else {
    //         Err(UrbitAPIError::FailedToCreateGraphInShip(
    //             graph_resource_name.to_string(),
    //         ))
    //     }
    // }

    /// Acquire a graph from Graph Store
    pub fn get_graph(&mut self, resource_ship: &str, resource_name: &str) -> Result<Graph> {
        let path = format!("/graph/{}/{}", resource_ship, resource_name);
        let res = self
            .channel
            .ship_interface
            .scry("graph-store", &path, "json")?;

        // If successfully acquired graph json
        if res.status().as_u16() == 200 {
            if let Ok(body) = res.text() {
                if let Ok(graph_json) = json::parse(&body) {
                    return Graph::from_json(graph_json);
                }
            }
        }
        // Else return error
        Err(UrbitAPIError::FailedToGetGraph(resource_name.to_string()))
    }

    /// Acquire a subset of a graph from Graph Store by specifying the start and end indices
    /// of the subset of the graph.
    pub fn get_graph_subset(
        &mut self,
        resource_ship: &str,
        resource_name: &str,
        start_index: &str,
        end_index: &str,
    ) -> Result<Graph> {
        let path = format!(
            "/graph-subset/{}/{}/{}/{}",
            resource_ship, resource_name, end_index, start_index
        );
        let res = self
            .channel
            .ship_interface
            .scry("graph-store", &path, "json")?;

        // If successfully acquired graph json
        if res.status().as_u16() == 200 {
            if let Ok(body) = res.text() {
                if let Ok(graph_json) = json::parse(&body) {
                    return Graph::from_json(graph_json);
                }
            }
        }
        // Else return error
        Err(UrbitAPIError::FailedToGetGraph(resource_name.to_string()))
    }

    /// Delete graph from Graph Store
    pub fn delete_graph(&mut self, resource_ship: &str, resource_name: &str) -> Result<()> {
        let prepped_json = object! {
            "delete": {
                "resource": {
                    "ship": resource_ship,
                    "name": resource_name
                }
            }
        };

        let resp =
            (&mut self.channel).poke("graph-view-action", "graph-update-2", &prepped_json)?;

        if resp.status().as_u16() == 204 {
            Ok(())
        } else {
            return Err(UrbitAPIError::FailedToRemoveGraphFromGraphStore(
                resource_name.to_string(),
            ));
        }
    }

    /// Leave graph in Graph Store
    pub fn leave_graph(&mut self, resource_ship: &str, resource_name: &str) -> Result<()> {
        let prepped_json = object! {
            "leave": {
                "resource": {
                    "ship": resource_ship,
                    "name": resource_name
                }
            }
        };

        let resp =
            (&mut self.channel).poke("graph-view-action", "graph-update-2", &prepped_json)?;

        if resp.status().as_u16() == 204 {
            Ok(())
        } else {
            return Err(UrbitAPIError::FailedToRemoveGraphFromGraphStore(
                resource_name.to_string(),
            ));
        }
    }

    /// Archive a graph in Graph Store
    pub fn archive_graph(&mut self, resource_ship: &str, resource_name: &str) -> Result<String> {
        let path = format!("/archive/{}/{}", resource_ship, resource_name);
        let res = self
            .channel
            .ship_interface
            .scry("graph-store", &path, "json")?;

        if res.status().as_u16() == 200 {
            if let Ok(body) = res.text() {
                return Ok(body);
            }
        }
        return Err(UrbitAPIError::FailedToArchiveGraph(
            resource_name.to_string(),
        ));
    }

    /// Unarchive a graph in Graph Store
    pub fn unarchive_graph(&mut self, resource_ship: &str, resource_name: &str) -> Result<String> {
        let path = format!("/unarchive/{}/{}", resource_ship, resource_name);
        let res = self
            .channel
            .ship_interface
            .scry("graph-store", &path, "json")?;

        if res.status().as_u16() == 200 {
            if let Ok(body) = res.text() {
                return Ok(body);
            }
        }
        return Err(UrbitAPIError::FailedToArchiveGraph(
            resource_name.to_string(),
        ));
    }

    /// Add a tag to a graph
    pub fn add_tag(&mut self, resource_ship: &str, resource_name: &str, tag: &str) -> Result<()> {
        let prepped_json = object! {
            "add-tag": {
                "resource": {
                    "ship": resource_ship,
                    "name": resource_name
                },
                "term":  tag
                }
        };

        let resp = (&mut self.channel).poke("graph-push-hook", "graph-update-2", &prepped_json)?;

        if resp.status().as_u16() == 204 {
            Ok(())
        } else {
            return Err(UrbitAPIError::FailedToAddTag(resource_name.to_string()));
        }
    }

    /// Remove a tag from a graph
    pub fn remove_tag(
        &mut self,
        resource_ship: &str,
        resource_name: &str,
        tag: &str,
    ) -> Result<()> {
        let prepped_json = object! {
            "remove-tag": {
                "resource": {
                    "ship": resource_ship,
                    "name": resource_name
                },
                "term":  tag
                }
        };

        let resp = (&mut self.channel).poke("graph-push-hook", "graph-update-2", &prepped_json)?;

        if resp.status().as_u16() == 204 {
            Ok(())
        } else {
            return Err(UrbitAPIError::FailedToRemoveTag(resource_name.to_string()));
        }
    }

    /// Performs a scry to get all keys
    pub fn get_keys(&mut self) -> Result<Vec<JsonValue>> {
        let resp = self
            .channel
            .ship_interface
            .scry("graph-store", "/keys", "json")?;

        if resp.status().as_u16() == 200 {
            let json_text = resp.text()?;
            if let Ok(json) = json::parse(&json_text) {
                let keys = json["graph-update"]["keys"].clone();
                let mut keys_list = vec![];
                for key in keys.members() {
                    keys_list.push(key.clone())
                }
                return Ok(keys_list);
            }
        }
        return Err(UrbitAPIError::FailedToFetchKeys);
    }

    /// Performs a scry to get all tags
    pub fn get_tags(&mut self) -> Result<Vec<JsonValue>> {
        let resp = self
            .channel
            .ship_interface
            .scry("graph-store", "/tags", "json")?;

        if resp.status().as_u16() == 200 {
            let json_text = resp.text()?;
            if let Ok(json) = json::parse(&json_text) {
                let tags = json["graph-update"]["tags"].clone();
                let mut tags_list = vec![];
                for tag in tags.members() {
                    tags_list.push(tag.clone())
                }
                return Ok(tags_list);
            }
        }
        return Err(UrbitAPIError::FailedToFetchTags);
    }

    /// Performs a scry to get all tags
    pub fn get_tag_queries(&mut self) -> Result<Vec<JsonValue>> {
        let resp = self
            .channel
            .ship_interface
            .scry("graph-store", "/tag-queries", "json")?;

        if resp.status().as_u16() == 200 {
            let json_text = resp.text()?;
            if let Ok(json) = json::parse(&json_text) {
                let tags = json["graph-update"]["tag-queries"].clone();
                let mut tags_list = vec![];
                for tag in tags.members() {
                    tags_list.push(tag.clone())
                }
                return Ok(tags_list);
            }
        }
        return Err(UrbitAPIError::FailedToFetchTags);
    }

    /// Acquire the time the update log of a given resource was last updated
    pub fn peek_update_log(&mut self, resource_ship: &str, resource_name: &str) -> Result<String> {
        let path = format!("/peek-update-log/{}/{}", resource_ship, resource_name);
        let res = self
            .channel
            .ship_interface
            .scry("graph-store", &path, "json")?;

        // If successfully acquired node json
        if res.status().as_u16() == 200 {
            if let Ok(body) = res.text() {
                return Ok(body);
            }
        }
        // Else return error
        Err(UrbitAPIError::FailedToGetGraph(resource_name.to_string()))
    }

    /// Acquire the update log for a given resource
    pub fn get_update_log(&mut self, resource_ship: &str, resource_name: &str) -> Result<String> {
        let path = format!("/update-log/{}/{}", resource_ship, resource_name);
        let res = self
            .channel
            .ship_interface
            .scry("graph-store", &path, "json")?;

        // If successfully acquired node json
        if res.status().as_u16() == 200 {
            if let Ok(body) = res.text() {
                return Ok(body);
            }
        }
        // Else return error
        Err(UrbitAPIError::FailedToGetGraph(resource_name.to_string()))
    }

    /// Acquire a subset of the update log for a given resource
    pub fn get_update_log_subset(
        &mut self,
        resource_ship: &str,
        resource_name: &str,
        start_index: &str,
        end_index: &str,
    ) -> Result<String> {
        let path = format!(
            "/update-log-subset/{}/{}/{}/{}",
            resource_ship, resource_name, end_index, start_index
        );
        let res = self
            .channel
            .ship_interface
            .scry("graph-store", &path, "json")?;

        // If successfully acquired node json
        if res.status().as_u16() == 200 {
            if let Ok(body) = res.text() {
                return Ok(body);
            }
        }
        // Else return error
        Err(UrbitAPIError::FailedToGetUpdateLog(
            resource_name.to_string(),
        ))
    }
}

pub fn module_to_validator_string(module: &Module) -> String {
    match module {
        Module::Chat => "graph-validator-chat".to_string(),
        Module::Notebook => "graph-validator-publish".to_string(),
        Module::Collection => "graph-validator-link".to_string(),
        Module::Null => "".to_string(),
    }
}

pub fn module_to_mark(module: &Module) -> String {
    match module {
        Module::Chat => "chat".to_string(),
        Module::Notebook => "publish".to_string(),
        Module::Collection => "link".to_string(),
        Module::Null => "".to_string(),
    }
}
