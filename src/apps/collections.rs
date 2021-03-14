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
        let mut comments: Vec<Comment> = vec![];
        // Check the to see if the children exist
        if node.children.len() > 0 && node.children[0].children.len() > 0 {
            // Find the latest revision of each of the comments
            for comment_node in &node.children {
                let mut latest_comment_revision_node = comment_node.children[0].clone();
                for revision_node in &comment_node.children {
                    if revision_node.index_tail() > latest_comment_revision_node.index_tail() {
                        latest_comment_revision_node = revision_node.clone();
                    }
                }
                comments.push(Comment::from_node(&latest_comment_revision_node));
            }
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
    /// Extracts a Collection's graph from the connected ship and parses it into a vector of `Link`s.
    pub fn export_collection(
        &mut self,
        collection_ship: &str,
        collection_name: &str,
    ) -> Result<Vec<Link>> {
        let graph = &self
            .channel
            .graph_store()
            .get_graph(collection_ship, collection_name)?;

        // Parse each top level node (Link) in the collection graph
        let mut links = vec![];
        for node in &graph.nodes {
            let link = Link::from_node(node)?;
            links.push(link);
        }

        Ok(links)
    }

    /// Adds a new link to the specified Collection that your ship has access to.
    /// Returns the index of the link.
    pub fn add_link(
        &mut self,
        collection_ship: &str,
        collection_name: &str,
        title: &str,
        url: &str,
    ) -> Result<String> {
        let mut gs = self.channel.graph_store();

        let link_contents = NodeContents::new().add_text(title).add_url(url);
        let link_node = gs.new_node(&link_contents);

        if let Ok(_) = gs.add_node(collection_ship, collection_name, &link_node) {
            Ok(link_node.index)
        } else {
            Err(UrbitAPIError::FailedToCreateNote(
                link_node.to_json().dump(),
            ))
        }
    }
}
