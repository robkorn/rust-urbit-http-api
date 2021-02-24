use crate::comment::Comment;
use crate::graph::NodeContents;
use crate::{Channel, Node, Result, UrbitAPIError};

/// A struct that provides an interface for interacting with Urbit notebooks
pub struct Notebook<'a> {
    pub channel: &'a mut Channel,
}

/// A struct that represents a Note from a Notebook
#[derive(Clone, Debug)]
pub struct Note {
    pub title: String,
    pub author: String,
    pub time_sent: String,
    /// The contents of the latest revision of the Note
    pub contents: NodeContents,
    // pub old_revisions_content: Vec<NodeContents>
    pub comments: Vec<Comment>,
}

impl Note {
    /// Create a new `Note`
    pub fn new(
        title: &str,
        author: &str,
        time_sent: &str,
        contents: &NodeContents,
        comments: &Vec<Comment>,
    ) -> Note {
        Note {
            title: title.to_string(),
            author: author.to_string(),
            time_sent: time_sent.to_string(),
            contents: contents.clone(),
            comments: comments.clone(),
        }
    }

    /// Convert from a `Node` to a `Note`
    pub fn from_node(node: &Node) -> Result<Note> {
        let mut comments: Vec<Comment> = vec![];
        // Find the comments node which has an index tail of `2`
        let comments_node = node
            .children
            .iter()
            .find(|c| c.index_tail() == "2")
            .ok_or(UrbitAPIError::InvalidNoteGraphNode(node.to_json().dump()))?;
        // Find the note content node (which holds all revisions as children) which has an index tail of `1`
        let content_node = node
            .children
            .iter()
            .find(|c| c.index_tail() == "1")
            .ok_or(UrbitAPIError::InvalidNoteGraphNode(node.to_json().dump()))?;

        for comment_node in &comments_node.children {
            for child in &comment_node.children {
                comments.push(Comment::from_node(child));
            }
        }

        // Find the latest revision of the notebook content
        let mut latest_revision_node = content_node.children[0].clone();
        for revision_node in &content_node.children {
            if revision_node.index_tail() > latest_revision_node.index_tail() {
                latest_revision_node = revision_node.clone()
            }
        }
        // Acquire the title, which is the first item in the revision node of the note
        let title = format!("{}", latest_revision_node.contents.content_list[0]["text"]);
        // Recreate the revision node contents with the title removed
        let contents =
            NodeContents::from_json(latest_revision_node.contents.content_list[1..].to_vec());
        let author = node.author.clone();
        let time_sent = node.time_sent_formatted();

        // Create the note
        Ok(Note::new(&title, &author, &time_sent, &contents, &comments))
    }

    /// Convert the contents of the latest revision of the Note to
    /// a series of markdown `String`s
    pub fn content_as_markdown(&self) -> Vec<String> {
        let formatted_string = self.contents.to_formatted_string();
        formatted_string
            .split("\\n")
            .map(|l| l.to_string())
            .collect()
    }
}

impl<'a> Notebook<'a> {
    /// Extracts a Notebook's graph from the connected ship and parses it into a vector of `Note`s
    pub fn export_notebook(
        &mut self,
        notebook_ship: &str,
        notebook_name: &str,
    ) -> Result<Vec<Note>> {
        let graph = &self
            .channel
            .graph_store()
            .get_graph(notebook_ship, notebook_name)?;

        // Parse each top level node (Note) in the notebook graph
        let mut notes = vec![];
        for node in &graph.nodes {
            let note = Note::from_node(node)?;
            notes.push(note);
        }

        Ok(notes)
    }
}
