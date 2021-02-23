use crate::graph::NodeContents;

/// A struct representing a comment either on a `Note` or on a collections `Link`
pub struct Comment {
    pub author: String,
    pub content: NodeContents,
    pub timestamp: String,
}
