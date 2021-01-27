use crate::error::{Result, UrbitAPIError};
use json::{object, JsonValue};
use regex::Regex;

/// Struct which represents a graph in Graph Store.
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
    pub contents: Vec<String>,
    pub hash: Option<String>,
    pub children: Vec<Node>,
}

impl Graph {
    // Create a new `Graph`
    pub fn new(nodes: Vec<Node>) -> Graph {
        Graph { nodes: nodes }
    }

    // Convert from graph `JsonValue` to `Graph`
    pub fn from_json(graph_json: JsonValue) -> Result<Graph> {
        let mut nodes = vec![];
        // Get the graph inner json
        let graph_text = format!("{}", graph_json["graph-update"]["add-graph"]["graph"]);

        // Create regex to capture each node json
        let re = Regex::new(r#""\d+":(.+?children":.+?})"#).unwrap();
        // For each capture group, create a node and add to to the nodes list
        for capture in re.captures_iter(&graph_text) {
            let json_string = capture
                .get(1)
                .ok_or(UrbitAPIError::FailedToCreateGraphFromJSON)?
                .as_str();
            let node_json =
                json::parse(json_string).map_err(|_| UrbitAPIError::FailedToCreateGraphFromJSON)?;
            let node = Node::from_json(&node_json)?;
            nodes.push(node);
        }
        Ok(Graph::new(nodes))
    }

    // Converts to `JsonValue`
    pub fn to_json(&self) -> JsonValue {
        let nodes_json: Vec<JsonValue> = self.nodes.iter().map(|n| n.to_json()).collect();
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
        contents: Vec<String>,
        hash: Option<String>,
        children: Vec<Node>,
    ) -> Node {
        Node {
            index: index,
            author: author,
            time_sent: time_sent,
            signatures: signatures,
            contents: contents,
            hash: hash,
            children: children,
        }
    }

    // Convert from node `JsonValue` to `Node`
    pub fn from_json(json: &JsonValue) -> Result<Node> {
        // Process all of the json fields
        let children = json["children"].clone();
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
            contents.push(
                content
                    .as_str()
                    .ok_or(UrbitAPIError::FailedToCreateGraphNodeFromJSON)?
                    .to_string(),
            );
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
            children: vec![],
        })
    }

    // Converts to `JsonValue`
    pub fn to_json(&self) -> JsonValue {
        let mut node_json = object!();
        node_json[self.index.clone()] = object! {
                        "post": {
                            "author": self.author.clone(),
                            "index": self.index.clone(),
                            "time-sent": self.time_sent,
                            "contents": self.contents.clone(),
                            "hash": null,
                            "signatures": []
                        },
                        "children": null
        };
        node_json
    }
}
