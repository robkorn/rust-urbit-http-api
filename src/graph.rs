use crate::error::{Result, UrbitAPIError};
use json::{object, JsonValue};
use regex::Regex;

/// Struct which represents a graph in Graph Store
/// as a list of Nodes. Simplistic implementation
/// may be updated in the future if performance becomes
/// inadequate in real world use cases.
#[derive(Clone, Debug)]
pub struct Graph {
    pub nodes: Vec<Node>,
}

/// Struct which represents a node in a graph in Graph Store
#[derive(Clone, Debug)]
pub struct Node {
    pub index: String,
    pub author: String,
    pub time_sent: u64,
    pub signatures: Vec<String>,
    pub contents: Vec<JsonValue>,
    pub hash: Option<String>,
}

impl Graph {
    /// Create a new `Graph`
    pub fn new(nodes: Vec<Node>) -> Graph {
        Graph { nodes: nodes }
    }

    /// Insert a `Node` into the `Graph`.
    /// Reads the index of the node and embeds it within
    /// the correct parent node if it is a child.
    pub fn insert(&mut self, node: Node) {
        self.nodes.push(node);
    }

    /// Attempts to find the parent of a given node within `self`
    /// with a naive linear search.
    pub fn find_node_parent(&self, child: &Node) -> Option<&Node> {
        self.nodes.iter().find(|n| n.is_direct_parent(&child))
    }

    /// Convert from graph `JsonValue` to `Graph`
    pub fn from_json(graph_json: JsonValue) -> Result<Graph> {
        // Create a new empty graph to insert nodes into
        let mut graph = Graph::new(vec![]);
        // Create a list of nodes all stripped of child associations
        let mut childless_nodes = vec![];
        // Get the graph inner json
        let graph_text = format!("{}", graph_json["graph-update"]["add-graph"]["graph"]);
        println!("graph text: {}", graph_text);

        // Create regex to capture each node json
        let re = Regex::new(r#""\d+":(.+?children":).+?"#)
            .map_err(|_| UrbitAPIError::FailedToCreateGraphFromJSON)?;
        // For each capture group, create a childless node
        for capture in re.captures_iter(&graph_text) {
            // Get the node json string without it's children
            let node_string = capture
                .get(1)
                .ok_or(UrbitAPIError::FailedToCreateGraphFromJSON)?
                .as_str()
                .to_string()
                + r#"null}"#;
            println!(" ");
            println!("node string: {}", node_string);
            let json = json::parse(&node_string)
                .map_err(|_| UrbitAPIError::FailedToCreateGraphNodeFromJSON)?;
            let processed_node = Node::from_json(&json)?;
            childless_nodes.push(processed_node);
        }

        // Insert all of the childless nodes into the graph.
        // Places them under the correct parent as required.
        for cn in childless_nodes {
            graph.insert(cn);
        }

        Ok(graph)
    }

    // Converts to `JsonValue`
    pub fn to_json(&self) -> JsonValue {
        let nodes_json: Vec<JsonValue> = self.nodes.iter().map(|n| n.to_json(None)).collect();
        object! {
                            "graph-update": {
                                "add-graph": {
                                    "graph": nodes_json,
        }
                }
                        }
    }
}

impl Node {
    // Create a new `Node`
    pub fn new(
        index: String,
        author: String,
        time_sent: u64,
        signatures: Vec<String>,
        contents: Vec<JsonValue>,
        hash: Option<String>,
    ) -> Node {
        Node {
            index: index,
            author: author,
            time_sent: time_sent,
            signatures: signatures,
            contents: contents,
            hash: hash,
        }
    }

    /// Extract the `Node`'s parent's index (if parent exists)
    pub fn parent_index(&self) -> Option<String> {
        let rev_index = self.index.chars().rev().collect::<String>();
        let split_index: Vec<&str> = rev_index.splitn(2, "/").collect();
        // Error check
        if split_index.len() < 2 {
            return None;
        }

        let parent_index = split_index[1].chars().rev().collect::<String>();

        Some(parent_index)
    }

    /// Check if a self is the direct parent of another `Node`.
    pub fn is_direct_parent(&self, potential_child: &Node) -> bool {
        if let Some(index) = potential_child.parent_index() {
            return self.index == index;
        }
        false
    }

    /// Check if self is a parent (direct or indirect) of another `Node`
    pub fn is_parent(&self, potential_child: &Node) -> bool {
        let pc_split_index: Vec<&str> = potential_child.index.split("/").collect();
        let parent_split_index: Vec<&str> = self.index.split("/").collect();

        // Verify the parent has a smaller split index than child
        if parent_split_index.len() > pc_split_index.len() {
            return false;
        }

        // Check if every index split part of the parent is part of
        // the child
        let mut matching = false;
        for n in 0..parent_split_index.len() - 1 {
            matching = parent_split_index[n] == pc_split_index[n]
        }

        // Return if parent index is fully part of the child index
        matching
    }

    /// Convert from node `JsonValue` to `Node`
    /// Defaults to no children.
    fn from_json(json: &JsonValue) -> Result<Node> {
        // Process all of the json fields
        let _children = json["children"].clone();
        let post_json = json["post"].clone();
        let index = post_json["index"]
            .as_str()
            .ok_or(UrbitAPIError::FailedToCreateGraphNodeFromJSON)?;
        let author = post_json["author"]
            .as_str()
            .ok_or(UrbitAPIError::FailedToCreateGraphNodeFromJSON)?;
        let time_sent = post_json["time-sent"]
            .as_u64()
            .ok_or(UrbitAPIError::FailedToCreateGraphNodeFromJSON)?;

        // Wrap hash in an Option for null case
        let hash = match post_json["hash"].is_null() {
            true => None,
            false => Some(
                post_json["hash"]
                    .as_str()
                    .ok_or(UrbitAPIError::FailedToCreateGraphNodeFromJSON)?
                    .to_string(),
            ),
        };

        // Convert array JsonValue to vector for contents
        let mut contents = vec![];
        for content in post_json["contents"].members() {
            contents.push(content.clone());
        }

        // Convert array JsonValue to vector for signatures
        let mut signatures = vec![];
        for signature in post_json["signatures"].members() {
            signatures.push(
                signature
                    .as_str()
                    .ok_or(UrbitAPIError::FailedToCreateGraphNodeFromJSON)?
                    .to_string(),
            );
        }

        Ok(Node {
            index: index.to_string(),
            author: author.to_string(),
            time_sent: time_sent,
            signatures: signatures,
            contents: contents,
            hash: hash,
        })
    }

    /// Converts to `JsonValue`
    pub fn to_json(&self, children: Option<JsonValue>) -> JsonValue {
        let mut node_json = object!();

        let final_children = match children {
            Some(json) => json,
            None => JsonValue::Null,
        };

        node_json[self.index.clone()] = object! {
                        "post": {
                            "author": self.author.clone(),
                            "index": self.index.clone(),
                            "time-sent": self.time_sent,
                            "contents": self.contents.clone(),
                            "hash": null,
                            "signatures": []
                        },
                        "children": final_children
        };
        node_json
    }
}
