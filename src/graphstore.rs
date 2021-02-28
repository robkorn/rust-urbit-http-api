use crate::graph::{Graph, Node, NodeContents};
use crate::helper::{get_current_da_time, get_current_time, index_dec_to_ud};
use crate::{Channel, Result, UrbitAPIError};
use json::object;

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

    /// Create a new Graph Store node using a specified index and creation time, and connected ship as author
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
