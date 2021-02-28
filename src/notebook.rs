use crate::comment::Comment;
use crate::graph::NodeContents;
use crate::helper::{get_current_da_time, get_current_time};
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
    pub contents: String,
    pub comments: Vec<Comment>,
    pub index: String,
}

/// An internal helper struct for analysing Notebook node indices
#[derive(Clone, Debug)]
struct NotebookIndex<'a> {
    pub index: &'a str,
    pub index_split: Vec<&'a str>,
}

impl Note {
    /// Create a new `Note`
    pub fn new(
        title: &str,
        author: &str,
        time_sent: &str,
        contents: &str,
        comments: &Vec<Comment>,
        index: &str,
    ) -> Note {
        Note {
            title: title.to_string(),
            author: author.to_string(),
            time_sent: time_sent.to_string(),
            contents: contents.to_string(),
            comments: comments.clone(),
            index: index.to_string(),
        }
    }

    /// Convert from a `Node` to a `Note`
    pub fn from_node(node: &Node, revision: Option<String>) -> Result<Note> {
        let mut comments: Vec<Comment> = vec![];
        // Find the comments node which has an index tail of `2`
        let comments_node = node
            .children
            .iter()
            .find(|c| c.index_tail() == "2")
            .ok_or(UrbitAPIError::InvalidNoteGraphNode(node.to_json().dump()))?;
        // Find the note content node which has an index tail of `1`
        let content_node = node
            .children
            .iter()
            .find(|c| c.index_tail() == "1")
            .ok_or(UrbitAPIError::InvalidNoteGraphNode(node.to_json().dump()))?;

        // Find the latest revision of each of the notebook comments
        for comment_node in &comments_node.children {
            let mut latest_comment_revision_node = comment_node.children[0].clone();
            for revision_node in &comment_node.children {
                if revision_node.index_tail() > latest_comment_revision_node.index_tail() {
                    latest_comment_revision_node = revision_node.clone();
                }
            }
            comments.push(Comment::from_node(&latest_comment_revision_node));
        }

        let mut fetched_revision_node = content_node.children[0].clone();

        match revision {
            Some(idx) => {
                // find a specific revision of the notebook content
                for revision_node in &content_node.children {
                    if revision_node.index == idx {
                        fetched_revision_node = revision_node.clone();
                    }
                }
            }
            None => {
                // Find the latest revision of the notebook content
                for revision_node in &content_node.children {
                    if revision_node.index_tail() > fetched_revision_node.index_tail() {
                        fetched_revision_node = revision_node.clone();
                    }
                }
            }
        }
        // Acquire the title, which is the first item in the revision node of the note
        let title = format!("{}", fetched_revision_node.contents.content_list[0]["text"]);
        // Acquire the note body, which is all in the second item in the revision node of the note
        let contents = format!("{}", fetched_revision_node.contents.content_list[1]["text"]);
        let author = fetched_revision_node.author.clone();
        let time_sent = fetched_revision_node.time_sent_formatted();

        // Create the note
        Ok(Note::new(
            &title,
            &author,
            &time_sent,
            &contents,
            &comments,
            &fetched_revision_node.index,
        ))
    }

    /// Convert the contents of the latest revision of the Note to
    /// a series of markdown `String`s
    pub fn content_as_markdown(&self) -> Vec<String> {
        let formatted_string = self.contents.clone();
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
            let note = Note::from_node(node, None)?;
            notes.push(note);
        }

        Ok(notes)
    }

    /// Fetch a note object given an index `note_index`. This note index can be the root index of the note
    /// or any of the child indexes of the note. If a child index for a specific revision of the note is passed
    /// then that revision will be fetched, otherwise latest revision is the default.
    pub fn fetch_note(
        &mut self,
        notebook_ship: &str,
        notebook_name: &str,
        note_index: &str,
    ) -> Result<Note> {
        // check index
        let index = NotebookIndex::new(note_index);
        if !index.is_valid() {
            return Err(UrbitAPIError::InvalidNoteGraphNodeIndex(
                note_index.to_string(),
            ));
        }

        // root note index
        let note_root_index = index.note_root_index();

        // get the note root node
        let node =
            &self
                .channel
                .graph_store()
                .get_node(notebook_ship, notebook_name, &note_root_index)?;
        let revision = match index.is_note_revision() {
            true => Some(note_index.to_string()),
            false => None,
        };

        return Ok(Note::from_node(node, revision)?);
    }

    /// Fetches the latest version of a note based on providing the index of a comment on said note.
    /// This is technically just a wrapper around `fetch_note`, but is implemented as a separate method
    /// to prevent overloading method meaning/documentation thereby preventing confusion.
    pub fn fetch_note_with_comment_index(
        &mut self,
        notebook_ship: &str,
        notebook_name: &str,
        comment_index: &str,
    ) -> Result<Note> {
        self.fetch_note(notebook_ship, notebook_name, comment_index)
    }

    /// Find the index of the latest revision of a note given an index `note_index`
    /// `note_index` can be any valid note index (even an index of a comment on the note)
    pub fn fetch_note_latest_revision_index(
        &mut self,
        notebook_ship: &str,
        notebook_name: &str,
        note_index: &str,
    ) -> Result<String> {
        // check index
        let index = NotebookIndex::new(note_index);
        if !index.is_valid() {
            return Err(UrbitAPIError::InvalidNoteGraphNodeIndex(
                note_index.to_string(),
            ));
        }

        // root note index
        let note_root_index = index.note_root_index();

        // get note root node
        let node =
            &self
                .channel
                .graph_store()
                .get_node(notebook_ship, notebook_name, &note_root_index)?;
        for pnode in &node.children {
            if pnode.index_tail() == "1" {
                let mut latestindex = NotebookIndex::new(&pnode.children[0].index);
                for rev in &pnode.children {
                    let revindex = NotebookIndex::new(&rev.index);
                    if revindex.index_tail() > latestindex.index_tail() {
                        latestindex = revindex.clone();
                    }
                }
                return Ok(latestindex.index.to_string());
            }
        }

        Err(UrbitAPIError::InvalidNoteGraphNodeIndex(
            note_index.to_string(),
        ))
    }

    /// Fetch a comment given an index `comment_index`.
    /// Index can be the comment root node index, or index of any revision.
    /// Will fetch most recent revision if passed root node index
    pub fn fetch_comment(
        &mut self,
        notebook_ship: &str,
        notebook_name: &str,
        comment_index: &str,
    ) -> Result<Comment> {
        // check index
        let index = NotebookIndex::new(comment_index);

        if !index.is_valid_comment_index() {
            return Err(UrbitAPIError::InvalidCommentGraphNodeIndex(
                comment_index.to_string(),
            ));
        }
        let comment_root_index = index.comment_root_index()?;

        // get comment root node
        let node = &self.channel.graph_store().get_node(
            notebook_ship,
            notebook_name,
            &comment_root_index,
        )?;

        if index.is_comment_root() {
            // find latest comment revision
            let mut newest = node.children[0].clone();
            for rnode in &node.children {
                if rnode.index_tail() > newest.index_tail() {
                    newest = rnode.clone();
                }
            }
            return Ok(Comment::from_node(&newest));
        } else {
            // find specific comment revision
            for rnode in &node.children {
                if rnode.index == comment_index {
                    return Ok(Comment::from_node(&rnode));
                }
            }
        }

        Err(UrbitAPIError::InvalidCommentGraphNodeIndex(
            comment_index.to_string(),
        ))
    }

    /// Fetch index of latest revision of a comment given an index `comment_index`.
    /// Index can be the comment root node index, or the index of any revision of the comment.
    pub fn fetch_comment_latest_revision_index(
        &mut self,
        notebook_ship: &str,
        notebook_name: &str,
        comment_index: &str,
    ) -> Result<String> {
        // check index
        let index = NotebookIndex::new(comment_index);

        if !index.is_valid_comment_index() {
            return Err(UrbitAPIError::InvalidCommentGraphNodeIndex(
                comment_index.to_string(),
            ));
        }
        let comment_root_index = index.comment_root_index()?;

        // get comment root node
        let node = &self.channel.graph_store().get_node(
            notebook_ship,
            notebook_name,
            &comment_root_index,
        )?;

        if node.children.len() > 0 {
            let mut newestindex = NotebookIndex::new(&node.children[0].index);
            for rnode in &node.children {
                let revindex = NotebookIndex::new(&rnode.index);
                if revindex.index_tail() > newestindex.index_tail() {
                    newestindex = revindex.clone();
                }
            }
            return Ok(newestindex.index.to_string());
        }

        Err(UrbitAPIError::InvalidCommentGraphNodeIndex(
            comment_index.to_string(),
        ))
    }

    /// Adds a new note to the notebook.
    /// Returns the index of the newly created first revision of the note.
    pub fn add_note(
        &mut self,
        notebook_ship: &str,
        notebook_name: &str,
        title: &str,
        body: &str,
    ) -> Result<String> {
        let mut gs = self.channel.graph_store();
        // make the root node for the note
        let node_root = gs.new_node(&NodeContents::new());
        // save creation time for other nodes
        let unix_time = node_root.time_sent;
        // index helper
        let index = NotebookIndex::new(&node_root.index);

        // make child 1 for note content
        // make child 2 for comments
        // make child 1/1 for initial note revision
        let node_root = node_root
            .add_child(&gs.new_node_specified(
                &index.note_content_node_index(),
                unix_time,
                &NodeContents::new(),
            ))
            .add_child(&gs.new_node_specified(
                &index.note_comments_node_index(),
                unix_time,
                &NodeContents::new(),
            ))
            .add_child(&gs.new_node_specified(
                &index.note_revision_index(1),
                unix_time,
                &NodeContents::new().add_text(title).add_text(body),
            ));

        if let Ok(_) = gs.add_node(notebook_ship, notebook_name, &node_root) {
            Ok(index.note_revision_index(1))
        } else {
            Err(UrbitAPIError::FailedToCreateNote(
                node_root.to_json().dump(),
            ))
        }
    }

    /// Update an existing note with a new title and body.
    /// `note_index` can be any valid note index.
    /// Returns index of the newly created revision.
    pub fn update_note(
        &mut self,
        notebook_ship: &str,
        notebook_name: &str,
        note_index: &str,
        title: &str,
        body: &str,
    ) -> Result<String> {
        // fetch latest revision of note (will return error if not a valid note index)
        let note_latest_index =
            self.fetch_note_latest_revision_index(notebook_ship, notebook_name, note_index)?;
        // index helper
        let index = NotebookIndex::new(&note_latest_index);
        // build new node index
        let note_new_index = index.next_revision_index()?;

        let mut gs = self.channel.graph_store();
        let unix_time = get_current_time();

        // add the node
        let node = gs.new_node_specified(
            &note_new_index,
            unix_time,
            &NodeContents::new().add_text(title).add_text(body),
        );

        if let Ok(_) = gs.add_node(notebook_ship, notebook_name, &node) {
            Ok(node.index.clone())
        } else {
            Err(UrbitAPIError::FailedToCreateNote(node.to_json().dump()))
        }
    }

    /// Add a new comment to a specific note inside of a notebook specified by `note_index`
    /// `note_index` can be any valid note/revision, and even the index of other comments.
    pub fn add_comment(
        &mut self,
        notebook_ship: &str,
        notebook_name: &str,
        note_index: &str,
        comment: &NodeContents,
    ) -> Result<String> {
        // check index
        let index = NotebookIndex::new(note_index);
        if !index.is_valid() {
            return Err(UrbitAPIError::InvalidNoteGraphNodeIndex(
                note_index.to_string(),
            ));
        }

        let mut gs = self.channel.graph_store();
        let unix_time = get_current_time();

        // make a new node under the note comments node  - this is root node for this comment
        let cmt_root_node = gs.new_node_specified(
            &index.new_comment_root_index(),
            unix_time,
            &NodeContents::new(),
        );
        // update index helper from new node
        let index = NotebookIndex::new(&cmt_root_node.index);
        // make initial comment revision node
        let cmt_rev_index = index.comment_revision_index(1)?;
        let cmt_rev_node = gs.new_node_specified(&cmt_rev_index, unix_time, comment);
        // assemble node tree
        let cmt_root_node = cmt_root_node.add_child(&cmt_rev_node);
        // add the nodes
        if let Ok(_) = gs.add_node(notebook_ship, notebook_name, &cmt_root_node) {
            Ok(cmt_rev_index.clone())
        } else {
            Err(UrbitAPIError::FailedToCreateComment(
                cmt_root_node.to_json().dump(),
            ))
        }
    }

    /// Update an existing comment on a note. `comment_index` must be a valid index for a comment
    /// for a note within the notebook specified which your ship has edit rights for.
    /// Returns index of the new comment revision
    pub fn update_comment(
        &mut self,
        notebook_ship: &str,
        notebook_name: &str,
        comment_index: &str,
        comment: &NodeContents,
    ) -> Result<String> {
        // fetch latest comment revision index (will return error if not a valid comment index)
        let cmt_latest_index =
            self.fetch_comment_latest_revision_index(notebook_ship, notebook_name, comment_index)?;
        // index helper
        let index = NotebookIndex::new(&cmt_latest_index);
        // build new node index
        let cmt_new_index = index.next_revision_index()?;

        // add the node
        let mut gs = self.channel.graph_store();
        let unix_time = get_current_time();

        let node = gs.new_node_specified(&cmt_new_index, unix_time, comment);

        if let Ok(_) = gs.add_node(notebook_ship, notebook_name, &node) {
            Ok(node.index.clone())
        } else {
            Err(UrbitAPIError::FailedToCreateComment(node.to_json().dump()))
        }
    }
}

impl<'a> NotebookIndex<'a> {
    /// Create a new `NotebookIndex`
    pub fn new(idx: &str) -> NotebookIndex {
        NotebookIndex {
            index: idx,
            index_split: idx.split("/").collect(),
        }
    }

    // notebook index slices
    // must have at least 2 slices to be valid notebook index
    // slice 0 must have len 0 - means index started with a "/"
    // slice 1 is note root node
    // slice 2 is "1" for note, "2" for comment
    // slice 3 is note revision or comment root node
    // slice 4 is comment revision

    /// is this any kind of valid notebook node index (comment or note)?
    pub fn is_valid(&self) -> bool {
        (self.index_split.len() >= 2) && (self.index_split[0].len() == 0)
    }

    /// is this the index of a note root node?
    pub fn is_note_root(&self) -> bool {
        (self.index_split.len() == 2) && (self.index_split[0].len() == 0)
    }

    /// is this the index of a specific note revision?
    pub fn is_note_revision(&self) -> bool {
        (self.index_split.len() == 4)
            && (self.index_split[0].len() == 0)
            && (self.index_split[2] == "1")
    }

    /// is this some kind of valid comment index?
    pub fn is_valid_comment_index(&self) -> bool {
        (self.index_split.len() >= 4)
            && (self.index_split[0].len() == 0)
            && (self.index_split[2] == "2")
    }

    /// is this the index of a comment root?
    pub fn is_comment_root(&self) -> bool {
        (self.index_split.len() == 4)
            && (self.index_split[0].len() == 0)
            && (self.index_split[2] == "2")
    }

    /// is this the index of a comment revision?
    pub fn is_comment_revision(&self) -> bool {
        (self.index_split.len() == 5)
            && (self.index_split[0].len() == 0)
            && (self.index_split[2] == "2")
    }

    /// root index of note
    pub fn note_root_index(&self) -> String {
        format!("/{}", self.index_split[1])
    }

    /// index of note content node, note revisions are children of this
    pub fn note_content_node_index(&self) -> String {
        format!("/{}/1", self.index_split[1])
    }

    /// index of note comments node, all note comments are children of this
    pub fn note_comments_node_index(&self) -> String {
        format!("/{}/2", self.index_split[1])
    }

    /// root index of comment (if this is a valid comment index)
    /// all revisions of a comment are children of the comment root
    pub fn comment_root_index(&self) -> Result<String> {
        if self.is_valid_comment_index() {
            Ok(format!(
                "/{}/2/{}",
                self.index_split[1], self.index_split[3]
            ))
        } else {
            Err(UrbitAPIError::InvalidCommentGraphNodeIndex(
                self.index.to_string(),
            ))
        }
    }
    /// generate a new comment root index using `get_current_da_time()`
    pub fn new_comment_root_index(&self) -> String {
        format!("/{}/2/{}", self.index_split[1], get_current_da_time())
    }

    /// str slice of final element of index
    pub fn index_tail(&self) -> &str {
        self.index_split[self.index_split.len() - 1]
    }

    /// revision number if this is index of a specific revision
    pub fn revision(&self) -> Result<u64> {
        if self.is_note_revision() {
            if let Ok(r) = self.index_split[3].parse::<u64>() {
                return Ok(r);
            }
        } else if self.is_comment_revision() {
            if let Ok(r) = self.index_split[4].parse::<u64>() {
                return Ok(r);
            }
        }

        Err(UrbitAPIError::InvalidNoteGraphNodeIndex(
            self.index.to_string(),
        ))
    }

    /// generates the index of next revision, if this is a valid note or comment revision index
    pub fn next_revision_index(&self) -> Result<String> {
        let rev = self.revision()?;
        let newrev = rev + 1;
        // we know index_split.len() is either 4 or 5 here as revision() was Ok
        if self.index_split.len() == 5 {
            Ok(format!(
                "/{}/2/{}/{}",
                self.index_split[1],
                self.index_split[3],
                &newrev.to_string()
            ))
        } else {
            Ok(format!(
                "/{}/1/{}",
                self.index_split[1],
                &newrev.to_string()
            ))
        }
    }

    /// generate a specific note revision index
    pub fn note_revision_index(&self, revision: u64) -> String {
        format!("/{}/1/{}", self.index_split[1], revision.to_string())
    }

    /// generate a specific comment revision index (if this is a valid comment index)
    pub fn comment_revision_index(&self, revision: u64) -> Result<String> {
        if self.is_valid_comment_index() {
            Ok(format!(
                "/{}/2/{}/{}",
                self.index_split[1],
                self.index_split[3],
                revision.to_string()
            ))
        } else {
            Err(UrbitAPIError::InvalidCommentGraphNodeIndex(
                self.index.to_string(),
            ))
        }
    }
}
