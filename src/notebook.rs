use crate::comment::Comment;
use crate::graph::NodeContents;
use crate::{Channel, Node, Result, UrbitAPIError};

/// A struct that provides an interface for interacting with Urbit notebooks
pub struct Notebook<'a> {
    pub channel: &'a mut Channel,
}

/// A struct that represents a Note from a notebook
pub struct Note {
    pub author: String,
    pub timestamp: String,
    pub content: NodeContents,
    pub comments: Vec<Comment>,
}

impl<'a> Notebook<'a> {}
