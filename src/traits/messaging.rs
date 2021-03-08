use crate::error::{Result, UrbitAPIError};
use crate::graph::{Node, NodeContents};
use crate::Channel;
use crossbeam::channel::{unbounded, Receiver};
use json::JsonValue;
use std::thread;
use std::time::Duration;

/// A struct that represents a message that is to be submitted to Urbit.
/// `Message` provides methods to build a message in chunks, thereby allowing you
/// to add content which needs to be parsed, for example links @p mentions.
/// It is technically an alias for the `NodeContents` struct.
pub type Message = NodeContents;

/// A `Message` with the author @p, post time and index also included
#[derive(Clone, Debug)]
pub struct AuthoredMessage {
    pub author: String,
    pub contents: Message,
    pub time_sent: String,
    pub index: String,
}

impl AuthoredMessage {
    /// Create a new `AuthoredMessage`
    pub fn new(author: &str, contents: &Message, time_sent: &str, index: &str) -> Self {
        AuthoredMessage {
            author: author.to_string(),
            contents: contents.clone(),
            time_sent: time_sent.to_string(),
            index: index.to_string(),
        }
    }
    /// Parses a `Node` into `Self`
    pub fn from_node(node: &Node) -> Self {
        Self::new(
            &node.author,
            &node.contents,
            &node.time_sent_formatted(),
            &node.index,
        )
    }

    /// Converts self into a human readable formatted string which
    /// includes the author, date, and node contents.
    pub fn to_formatted_string(&self) -> String {
        let content = self.contents.to_formatted_string();
        format!("{} - ~{}:{}", self.time_sent, self.author, content)
    }
}

/// A trait which wraps both chats & DMs.
pub trait Messaging {
    /// Returns the reference to the Channel being used
    fn channel(&mut self) -> &mut Channel;

    /// Send a message to an Urbit chat/DM.
    /// Returns the index of the node that was added to Graph Store.
    fn send_message(
        &mut self,
        resource_ship: &str,
        resource_name: &str,
        message: &Message,
    ) -> Result<String> {
        let node = self.channel().graph_store().new_node(message);

        if let Ok(_) = self
            .channel()
            .graph_store()
            .add_node(resource_ship, resource_name, &node)
        {
            Ok(node.index)
        } else {
            Err(UrbitAPIError::FailedToSendChatMessage(
                message.to_json().dump(),
            ))
        }
    }

    /// Extracts messages automatically into a list of formatted `String`s
    fn export_message_log(
        &mut self,
        resource_ship: &str,
        resource_name: &str,
    ) -> Result<Vec<String>> {
        let mut export_log = vec![];
        let authored_messages = self.export_authored_messages(resource_ship, resource_name)?;

        for am in authored_messages {
            if !am.contents.is_empty() {
                export_log.push(am.to_formatted_string());
            }
        }

        Ok(export_log)
    }

    /// Extracts messages as `AuthoredMessage`s
    fn export_authored_messages(
        &mut self,
        resource_ship: &str,
        resource_name: &str,
    ) -> Result<Vec<AuthoredMessage>> {
        let mut authored_messages = vec![];
        let nodes = self.export_message_nodes(resource_ship, resource_name)?;

        for node in nodes {
            if !node.contents.is_empty() {
                let authored_message = AuthoredMessage::from_node(&node);
                authored_messages.push(authored_message);
            }
        }

        Ok(authored_messages)
    }

    /// Extracts a message nodes
    fn export_message_nodes(
        &mut self,
        resource_ship: &str,
        resource_name: &str,
    ) -> Result<Vec<Node>> {
        let messages_graph = &self
            .channel()
            .graph_store()
            .get_graph(resource_ship, resource_name)?;

        let mut nodes = messages_graph.clone().nodes;
        nodes.sort_by(|a, b| a.time_sent.cmp(&b.time_sent));

        Ok(nodes)
    }

    /// Subscribe to and watch for messages. This method returns a `Receiver` with the
    /// `AuthoredMessage`s that are posted after subscribing. Simply call `receiver.try_recv()`
    /// to read the next `AuthoredMessage` if one has been posted.
    ///
    /// Technical Note: This method actually creates a new `Channel` with your Urbit Ship, and spawns a new unix thread
    /// locally that processes all messages on said channel. This is required due to borrowing mechanisms in Rust, however
    /// on the plus side this makes it potentially more performant by each subscription having it's own unix thread.
    fn subscribe_to_messages(
        &mut self,
        resource_ship: &str,
        resource_name: &str,
    ) -> Result<Receiver<AuthoredMessage>> {
        let resource_ship = resource_ship.to_string();
        let resource_name = resource_name.to_string();
        // Create sender/receiver
        let (s, r) = unbounded();
        // Creating a new Ship Interface Channel to pass into the new thread
        // to be used to communicate with the Urbit ship
        let mut new_channel = self.channel().ship_interface.create_channel()?;

        thread::spawn(move || {
            // Infinitely watch for new graph store updates
            let channel = &mut new_channel;
            channel
                .create_new_subscription("graph-store", "/updates")
                .ok();
            loop {
                channel.parse_event_messages();
                let res_graph_updates = &mut channel.find_subscription("graph-store", "/updates");
                if let Some(graph_updates) = res_graph_updates {
                    // Read all of the current SSE messages to find if any are for the resource
                    // we are looking for.
                    loop {
                        let pop_res = graph_updates.pop_message();
                        // Acquire the message
                        if let Some(mess) = &pop_res {
                            // Parse it to json
                            if let Ok(json) = json::parse(mess) {
                                // If the graph-store node update is not for the correct resource
                                // then continue to next message.
                                if !check_resource_json(&resource_ship, &resource_name, &json) {
                                    continue;
                                }
                                // Otherwise, parse json to a `Node`
                                if let Ok(node) = Node::from_graph_update_json(&json) {
                                    // Parse it as an `AuthoredMessage`
                                    let authored_message = AuthoredMessage::from_node(&node);
                                    let _ = s.send(authored_message);
                                }
                            }
                        }
                        // If no messages left, stop
                        if let None = &pop_res {
                            break;
                        }
                    }
                }
                // Pause for half a second
                thread::sleep(Duration::new(0, 500000000));
            }
        });
        Ok(r)
    }
}

/// Checks whether the resource json matches the resource_name & resource_ship
fn check_resource_json(
    resource_ship: &str,
    resource_name: &str,
    resource_json: &JsonValue,
) -> bool {
    let resource = resource_json["graph-update"]["add-nodes"]["resource"].clone();
    let json_resource_name = format!("{}", resource["name"]);
    let json_resource_ship = format!("~{}", resource["ship"]);
    if json_resource_name == resource_name && json_resource_ship == resource_ship {
        return true;
    }
    false
}
