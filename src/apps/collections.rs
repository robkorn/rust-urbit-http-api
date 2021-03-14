use crate::apps::notebook::Comment;
use crate::graph::NodeContents;
use crate::helper::{get_current_da_time, get_current_time};
use crate::AuthoredMessage;
use crate::{Channel, Node, Result, UrbitAPIError};

/// A struct that provides an interface for interacting with Urbit collections
pub struct Collection<'a> {
    pub channel: &'a mut Channel,
}

/// A struct that represents a Collection link
#[derive(Clone, Debug)]
pub struct Link {
    pub title: String,
    pub author: String,
    pub time_sent: String,
    pub url: String,
    pub comments: Vec<Comment>,
    pub index: String,
}

impl Link {
    /// Create a new `Link`
    pub fn new(
        title: &str,
        author: &str,
        time_sent: &str,
        url: &str,
        comments: &Vec<Comment>,
        index: &str,
    ) -> Link {
        Link {
            title: title.to_string(),
            author: author.to_string(),
            time_sent: time_sent.to_string(),
            url: url.to_string(),
            comments: comments.clone(),
            index: index.to_string(),
        }
    }

    /// Convert from a `Node` to a `Link`
    pub fn from_node(node: &Node) -> Result<Link> {
        println!("Node children: {:?}", &node.children);
        let mut comments: Vec<Comment> = vec![];
        // Find the comments node which has an index tail of `2`
        let comments_node = node
            .children
            .iter()
            .find(|c| c.index_tail() == "2")
            .ok_or(UrbitAPIError::InvalidLinkGraphNode(node.to_json().dump()))?;

        // Find the latest revision of each of the comments
        for comment_node in &comments_node.children {
            let mut latest_comment_revision_node = comment_node.children[0].clone();
            for revision_node in &comment_node.children {
                if revision_node.index_tail() > latest_comment_revision_node.index_tail() {
                    latest_comment_revision_node = revision_node.clone();
                }
            }
            comments.push(Comment::from_node(&latest_comment_revision_node));
        }

        // Acquire the title, which is the first item in the content_list
        let title = format!("{}", node.contents.content_list[0]["text"]);
        // Acquire the url, which is the second item in the content_list
        let url = format!("{}", node.contents.content_list[1]["url"]);
        let author = node.author.clone();
        let time_sent = node.time_sent_formatted();

        // Create the note
        Ok(Link::new(
            &title,
            &author,
            &time_sent,
            &url,
            &comments,
            &node.index,
        ))
    }
}

impl<'a> Collection<'a> {
    //     /// Extracts a Notebook's graph from the connected ship and parses it into a vector of `Note`s
    //     pub fn export_notebook(
    //         &mut self,
    //         notebook_ship: &str,
    //         notebook_name: &str,
    //     ) -> Result<Vec<Note>> {
    //         let graph = &self
    //             .channel
    //             .graph_store()
    //             .get_graph(notebook_ship, notebook_name)?;

    //         // Parse each top level node (Note) in the notebook graph
    //         let mut notes = vec![];
    //         for node in &graph.nodes {
    //             let note = Note::from_node(node, None)?;
    //             notes.push(note);
    //         }

    //         Ok(notes)
    //     }

    //     /// Fetch a note object given an index `note_index`. This note index can be the root index of the note
    //     /// or any of the child indexes of the note. If a child index for a specific revision of the note is passed
    //     /// then that revision will be fetched, otherwise latest revision is the default.
    //     pub fn fetch_note(
    //         &mut self,
    //         notebook_ship: &str,
    //         notebook_name: &str,
    //         note_index: &str,
    //     ) -> Result<Note> {
    //         // check index
    //         let index = NotebookIndex::new(note_index);
    //         if !index.is_valid() {
    //             return Err(UrbitAPIError::InvalidNoteGraphNodeIndex(
    //                 note_index.to_string(),
    //             ));
    //         }

    //         // root note index
    //         let note_root_index = index.note_root_index();

    //         // get the note root node
    //         let node =
    //             &self
    //                 .channel
    //                 .graph_store()
    //                 .get_node(notebook_ship, notebook_name, &note_root_index)?;
    //         let revision = match index.is_note_revision() {
    //             true => Some(note_index.to_string()),
    //             false => None,
    //         };

    //         return Ok(Note::from_node(node, revision)?);
    //     }

    //     /// Fetches the latest version of a note based on providing the index of a comment on said note.
    //     /// This is technically just a wrapper around `fetch_note`, but is implemented as a separate method
    //     /// to prevent overloading method meaning/documentation thereby preventing confusion.
    //     pub fn fetch_note_with_comment_index(
    //         &mut self,
    //         notebook_ship: &str,
    //         notebook_name: &str,
    //         comment_index: &str,
    //     ) -> Result<Note> {
    //         self.fetch_note(notebook_ship, notebook_name, comment_index)
    //     }

    //     /// Find the index of the latest revision of a note given an index `note_index`
    //     /// `note_index` can be any valid note index (even an index of a comment on the note)
    //     pub fn fetch_note_latest_revision_index(
    //         &mut self,
    //         notebook_ship: &str,
    //         notebook_name: &str,
    //         note_index: &str,
    //     ) -> Result<String> {
    //         // check index
    //         let index = NotebookIndex::new(note_index);
    //         if !index.is_valid() {
    //             return Err(UrbitAPIError::InvalidNoteGraphNodeIndex(
    //                 note_index.to_string(),
    //             ));
    //         }

    //         // root note index
    //         let note_root_index = index.note_root_index();

    //         // get note root node
    //         let node =
    //             &self
    //                 .channel
    //                 .graph_store()
    //                 .get_node(notebook_ship, notebook_name, &note_root_index)?;
    //         for pnode in &node.children {
    //             if pnode.index_tail() == "1" {
    //                 let mut latestindex = NotebookIndex::new(&pnode.children[0].index);
    //                 for rev in &pnode.children {
    //                     let revindex = NotebookIndex::new(&rev.index);
    //                     if revindex.index_tail() > latestindex.index_tail() {
    //                         latestindex = revindex.clone();
    //                     }
    //                 }
    //                 return Ok(latestindex.index.to_string());
    //             }
    //         }

    //         Err(UrbitAPIError::InvalidNoteGraphNodeIndex(
    //             note_index.to_string(),
    //         ))
    //     }

    //     /// Fetch a comment given an index `comment_index`.
    //     /// Index can be the comment root node index, or index of any revision.
    //     /// Will fetch most recent revision if passed root node index
    //     pub fn fetch_comment(
    //         &mut self,
    //         notebook_ship: &str,
    //         notebook_name: &str,
    //         comment_index: &str,
    //     ) -> Result<Comment> {
    //         // check index
    //         let index = NotebookIndex::new(comment_index);

    //         if !index.is_valid_comment_index() {
    //             return Err(UrbitAPIError::InvalidCommentGraphNodeIndex(
    //                 comment_index.to_string(),
    //             ));
    //         }
    //         let comment_root_index = index.comment_root_index()?;

    //         // get comment root node
    //         let node = &self.channel.graph_store().get_node(
    //             notebook_ship,
    //             notebook_name,
    //             &comment_root_index,
    //         )?;

    //         if index.is_comment_root() {
    //             // find latest comment revision
    //             let mut newest = node.children[0].clone();
    //             for rnode in &node.children {
    //                 if rnode.index_tail() > newest.index_tail() {
    //                     newest = rnode.clone();
    //                 }
    //             }
    //             return Ok(Comment::from_node(&newest));
    //         } else {
    //             // find specific comment revision
    //             for rnode in &node.children {
    //                 if rnode.index == comment_index {
    //                     return Ok(Comment::from_node(&rnode));
    //                 }
    //             }
    //         }

    //         Err(UrbitAPIError::InvalidCommentGraphNodeIndex(
    //             comment_index.to_string(),
    //         ))
    //     }

    //     /// Fetch index of latest revision of a comment given an index `comment_index`.
    //     /// Index can be the comment root node index, or the index of any revision of the comment.
    //     pub fn fetch_comment_latest_revision_index(
    //         &mut self,
    //         notebook_ship: &str,
    //         notebook_name: &str,
    //         comment_index: &str,
    //     ) -> Result<String> {
    //         // check index
    //         let index = NotebookIndex::new(comment_index);

    //         if !index.is_valid_comment_index() {
    //             return Err(UrbitAPIError::InvalidCommentGraphNodeIndex(
    //                 comment_index.to_string(),
    //             ));
    //         }
    //         let comment_root_index = index.comment_root_index()?;

    //         // get comment root node
    //         let node = &self.channel.graph_store().get_node(
    //             notebook_ship,
    //             notebook_name,
    //             &comment_root_index,
    //         )?;

    //         if node.children.len() > 0 {
    //             let mut newestindex = NotebookIndex::new(&node.children[0].index);
    //             for rnode in &node.children {
    //                 let revindex = NotebookIndex::new(&rnode.index);
    //                 if revindex.index_tail() > newestindex.index_tail() {
    //                     newestindex = revindex.clone();
    //                 }
    //             }
    //             return Ok(newestindex.index.to_string());
    //         }

    //         Err(UrbitAPIError::InvalidCommentGraphNodeIndex(
    //             comment_index.to_string(),
    //         ))
    //     }

    //     /// Adds a new note to the notebook.
    //     /// Returns the index of the newly created first revision of the note.
    //     pub fn add_note(
    //         &mut self,
    //         notebook_ship: &str,
    //         notebook_name: &str,
    //         title: &str,
    //         body: &str,
    //     ) -> Result<String> {
    //         let mut gs = self.channel.graph_store();
    //         // make the root node for the note
    //         let node_root = gs.new_node(&NodeContents::new());
    //         // save creation time for other nodes
    //         let unix_time = node_root.time_sent;
    //         // index helper
    //         let index = NotebookIndex::new(&node_root.index);

    //         // make child 1 for note content
    //         // make child 2 for comments
    //         // make child 1/1 for initial note revision
    //         let node_root = node_root
    //             .add_child(&gs.new_node_specified(
    //                 &index.note_content_node_index(),
    //                 unix_time,
    //                 &NodeContents::new(),
    //             ))
    //             .add_child(&gs.new_node_specified(
    //                 &index.note_comments_node_index(),
    //                 unix_time,
    //                 &NodeContents::new(),
    //             ))
    //             .add_child(&gs.new_node_specified(
    //                 &index.note_revision_index(1),
    //                 unix_time,
    //                 &NodeContents::new().add_text(title).add_text(body),
    //             ));

    //         if let Ok(_) = gs.add_node(notebook_ship, notebook_name, &node_root) {
    //             Ok(index.note_revision_index(1))
    //         } else {
    //             Err(UrbitAPIError::FailedToCreateNote(
    //                 node_root.to_json().dump(),
    //             ))
    //         }
    //     }
}
