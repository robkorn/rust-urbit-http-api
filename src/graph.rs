/// Struct which represents a graph in Graph Store
#[derive(Clone, Debug)]
pub struct Graph {
    pub nodes: Vec<Node>,
}

/// Struct which represents a node in a graph in Graph Store
#[derive(Clone, Debug)]
pub struct Node {
    pub index: String,
    pub author: String,
    pub time_sent: u128,
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
}

impl Node {
    // Create a new `Node`
    pub fn new(
        index: String,
        author: String,
        time_sent: u128,
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
}
