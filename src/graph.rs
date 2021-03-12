use crate::error::{Result, UrbitAPIError};
use chrono::prelude::*;
use json::{object, JsonValue};
use regex::Regex;

/// Struct which represents a graph in Graph Store
/// as a list of Nodes. Simplistic implementation
/// may be updated in the future if performance becomes
/// inadequate in real world use cases.
#[derive(Clone, Debug)]
pub struct Graph {
    /// List of nodes structured as a graph with children
    pub nodes: Vec<Node>,
}

/// Struct which represents a node in a graph in Graph Store
#[derive(Clone, Debug)]
pub struct Node {
    pub index: String,
    pub author: String,
    pub time_sent: u64,
    pub signatures: Vec<String>,
    pub contents: NodeContents,
    pub hash: Option<String>,
    pub children: Vec<Node>,
}

/// Struct which represents the contents inside of a node
#[derive(Debug, Clone)]
pub struct NodeContents {
    pub content_list: Vec<JsonValue>,
}

impl Graph {
    /// Create a new `Graph`
    pub fn new(nodes: Vec<Node>) -> Graph {
        Graph { nodes: nodes }
    }

    /// Insert a `Node` into the top level of the `Graph`.
    pub fn insert(&mut self, node: Node) {
        self.nodes.push(node);
    }

    /// Convert from graph `JsonValue` to `Graph`
    pub fn from_json(graph_json: JsonValue) -> Result<Graph> {
        // Create a new empty graph to insert nodes into
        let mut graph = Graph::new(vec![]);
        // Create a list of nodes all stripped of child associations
        let mut childless_nodes = vec![];
        // Get the graph inner json
        let mut graph_text = format!("{}", graph_json["graph-update"]["add-graph"]["graph"]);
        if graph_text == "null" {
            graph_text = format!("{}", graph_json["graph-update"]["add-nodes"]["nodes"]);
        }

        // Create regex to capture each node json
        let re = Regex::new(r#"\d+":(.+?children":).+?"#)
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
            let json = json::parse(&node_string)
                .map_err(|_| UrbitAPIError::FailedToCreateGraphNodeFromJSON)?;
            let processed_node = Node::from_json(&json)?;
            childless_nodes.push(processed_node);
        }

        // Failed to extract nodes from json via Regex
        if childless_nodes.len() == 0 {
            return Err(UrbitAPIError::FailedToCreateGraphFromJSON);
        }

        // Create a placeholder node that accumulates all of the children
        // before being added to the graph
        let mut building_node = childless_nodes[0].clone();
        // Insert all of the childless nodes into the graph
        // under the correct parent.
        for i in 1..childless_nodes.len() {
            if building_node.is_parent(&childless_nodes[i]) {
                // Add the child into the deepest depth possible and update building_node
                building_node = building_node.add_child(&childless_nodes[i]);
            } else {
                // Insert the finished `building_node` into the graph
                graph.insert(building_node.clone());
                building_node = childless_nodes[i].clone();
            }
        }
        // Add the final created `building_node` from the last
        // iteration of the for loop.
        graph.insert(building_node.clone());

        // Return the finished graph
        Ok(graph)
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
        contents: NodeContents,
        hash: Option<String>,
    ) -> Node {
        Node {
            index: index,
            author: author,
            time_sent: time_sent,
            signatures: signatures,
            contents: contents,
            hash: hash,
            children: vec![],
        }
    }

    /// Extract the node's final section (after the last `/`) of the index
    pub fn index_tail(&self) -> String {
        let split_index: Vec<&str> = self.index.split("/").collect();
        split_index[split_index.len() - 1].to_string()
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
        for n in 0..parent_split_index.len() {
            matching = parent_split_index[n] == pc_split_index[n]
        }

        // Return if parent index is fully part of the child index
        matching
    }

    /// Creates a copy of self and searches through the children to find
    /// the deepest depth which the `new_child` can be placed.
    pub fn add_child(&self, new_child: &Node) -> Node {
        let mut new_self = self.clone();
        for i in 0..self.children.len() {
            let child = &new_self.children[i];
            if child.is_direct_parent(new_child) {
                new_self.children[i].children.push(new_child.clone());
                return new_self;
            } else if child.is_parent(new_child) {
                new_self.children[i] = child.add_child(new_child);
                return new_self;
            }
        }
        new_self.children.push(new_child.clone());
        new_self
    }
    /// Formats the `time_sent` field to be human readable date-time in UTC
    pub fn time_sent_formatted(&self) -> String {
        let unix_time = self.time_sent as i64 / 1000;
        let date_time: DateTime<Utc> =
            DateTime::from_utc(NaiveDateTime::from_timestamp(unix_time, 0), Utc);
        let new_date = date_time.format("%Y-%m-%d %H:%M:%S");
        format!("{}", new_date)
    }

    /// Converts to `JsonValue`
    /// creates a json object with one field: key = the node's full index path, value = json representation of node
    pub fn to_json(&self) -> JsonValue {
        let mut node_json = object!();
        node_json[self.index.clone()] = self.to_json_value();
        node_json
    }

    /// Converts to `JsonValue`
    /// json representation of a node
    fn to_json_value(&self) -> JsonValue {
        let mut children = object!();
        for child in &self.children {
            children[child.index_tail()] = child.to_json_value();
        }

        let result_json = object! {
                        "post": {
                            "author": self.author.clone(),
                            "index": self.index.clone(),
                            "time-sent": self.time_sent,
                            "contents": self.contents.to_json(),
                            "hash": null,
                            "signatures": []
                        },
                        "children": children
        };
        result_json
    }

    /// Convert from node `JsonValue` which is wrapped up in a few wrapper fields
    /// into a `Node`, with children if they exist.
    pub fn from_graph_update_json(wrapped_json: &JsonValue) -> Result<Node> {
        let dumped = wrapped_json["graph-update"]["add-nodes"]["nodes"].dump();
        let split: Vec<&str> = dumped.splitn(2, ":").collect();
        if split.len() <= 1 {
            return Err(UrbitAPIError::FailedToCreateGraphNodeFromJSON);
        }

        let mut inner_string = split[1].to_string();
        inner_string.remove(inner_string.len() - 1);

        let inner_json = json::parse(&inner_string)
            .map_err(|_| UrbitAPIError::FailedToCreateGraphNodeFromJSON)?;

        Self::from_json(&inner_json)
    }

    /// Convert from straight node `JsonValue` to `Node`
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
        let mut json_contents = vec![];
        for content in post_json["contents"].members() {
            json_contents.push(content.clone());
        }
        let contents = NodeContents::from_json(json_contents);

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

        // children
        let mut node_children: Vec<Node> = vec![];
        if let JsonValue::Object(o) = children {
            for (_, val) in o.iter() {
                if let Ok(child_node) = Node::from_json(val) {
                    node_children.push(child_node);
                }
            }
        }

        Ok(Node {
            index: index.to_string(),
            author: author.to_string(),
            time_sent: time_sent,
            signatures: signatures,
            contents: contents,
            hash: hash,
            children: node_children,
        })
    }
}

/// Methods for `NodeContents`
impl NodeContents {
    /// Create a new empty `NodeContents`
    pub fn new() -> NodeContents {
        NodeContents {
            content_list: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.content_list.len() == 0
    }

    /// Appends text to the end of the list of contents
    pub fn add_text(&self, text: &str) -> NodeContents {
        let formatted = object! {
            "text": text
        };
        self.add_to_contents(formatted)
    }

    /// Appends a url to the end of the list of contents
    pub fn add_url(&self, url: &str) -> NodeContents {
        let formatted = object! {
            "url": url
        };
        self.add_to_contents(formatted)
    }

    /// Appends a mention to another @p/ship to the end of the list of contents
    pub fn add_mention(&self, referenced_ship: &str) -> NodeContents {
        let formatted = object! {
            "mention": referenced_ship
        };
        self.add_to_contents(formatted)
    }

    /// Appends a code block to the end of the list of contents
    pub fn add_code(&self, expression: &str, output: &str) -> NodeContents {
        let formatted = object! {
            "code": {
                "expression": expression,
                "output": [[output]]
            }
        };
        self.add_to_contents(formatted)
    }

    /// Create a `NodeContents` from a list of `JsonValue`s
    pub fn from_json(json_contents: Vec<JsonValue>) -> NodeContents {
        NodeContents {
            content_list: json_contents,
        }
    }

    /// Convert the `NodeContents` into a json array in a `JsonValue`
    pub fn to_json(&self) -> JsonValue {
        self.content_list.clone().into()
    }

    /// Convert the `NodeContents` into a single `String` that is formatted
    /// for human reading.
    pub fn to_formatted_string(&self) -> String {
        let mut result = "".to_string();
        for item in &self.content_list {
            // Convert item into text
            let text = Self::extract_content_text(item);
            result = result + " " + text.trim();
        }
        result
    }

    /// Converts the `NodeContents` into a `String` that is formatted
    /// for human reading, which is then split at every whitespace.
    /// Useful for parsing a message.
    pub fn to_formatted_words(&self) -> Vec<String> {
        let formatted_string = self.to_formatted_string();
        formatted_string
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }

    // Extracts content from a content list item `JsonValue`
    fn extract_content_text(json: &JsonValue) -> String {
        let mut result = "  ".to_string();
        if !json["text"].is_empty() {
            result = json["text"].dump();
        } else if !json["url"].is_empty() {
            result = json["url"].dump();
        } else if !json["mention"].is_empty() {
            result = json["mention"].dump();
            result.remove(0);
            result.remove(result.len() - 1);
            return format!("~{}", result);
        } else if !json["code"].is_empty() {
            result = json["code"].dump();
        }
        result.remove(0);
        result.remove(result.len() - 1);
        result
    }

    /// Internal method to append `JsonValue` to the end of the list of contents
    fn add_to_contents(&self, json: JsonValue) -> NodeContents {
        let mut contents = self.content_list.clone();
        contents.append(&mut vec![json]);
        NodeContents {
            content_list: contents,
        }
    }
}
