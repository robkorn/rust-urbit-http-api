use crate::graph::NodeContents;
use crate::{Channel, Node, Result, UrbitAPIError};
use crossbeam::channel::{unbounded, Receiver};
use json::JsonValue;
use std::thread;
use std::time::Duration;

/// A struct that provides an interface for interacting with Urbit chats
pub struct Chat<'a> {
    pub channel: &'a mut Channel,
}

/// A struct that represents a message that is to be sent to an Urbit chat.
/// `Message` provides methods to build a message in chunks, thereby allowing you
/// to add content which needs to be parsed, for example links @p mentions.
/// It is technically an alias for the `NodeContents` struct.
// #[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub type Message = NodeContents;

/// A `Message` with the author @p also included
#[derive(Clone, Debug)]
pub struct AuthoredMessage {
    pub author: String,
    pub message: Message,
}

impl AuthoredMessage {
    /// Create a new `AuthoredMessage`
    pub fn new(author: String, message: Message) -> Self {
        AuthoredMessage {
            author: author,
            message: message,
        }
    }
}

/// Methods for interacting with a Chat
impl<'a> Chat<'a> {
    /// Send a message to an Urbit chat.
    /// Returns the index of the node that was added
    /// to Graph Store.
    pub fn send_message(
        &mut self,
        chat_ship: &str,
        chat_name: &str,
        message: &Message,
    ) -> Result<String> {
        let node = self.channel.graph_store().new_node(message);

        if let Ok(_) = self
            .channel
            .graph_store()
            .add_node(chat_ship, chat_name, &node)
        {
            Ok(node.index)
        } else {
            Err(UrbitAPIError::FailedToSendChatMessage(
                message.to_json().dump(),
            ))
        }
    }

    /// Extracts a Chat's messages automatically into a list of `String`s
    pub fn export_chat_log(&mut self, chat_ship: &str, chat_name: &str) -> Result<Vec<String>> {
        let chat_graph = &self.channel.graph_store().get_graph(chat_ship, chat_name)?;
        let mut export_log = vec![];

        let mut nodes = chat_graph.clone().nodes;
        nodes.sort_by(|a, b| a.time_sent.cmp(&b.time_sent));

        for node in nodes {
            if !node.contents.is_empty() {
                export_log.push(node.to_formatted_string());
            }
        }

        Ok(export_log)
    }

    /// Subscribe to and watch for messages for a specific chat. This method returns a `Receiver` with the
    /// `AuthoredMessage`s that are posted to the chat after subscribing. Simply call `receiver.try_recv()`
    /// to read the next `AuthoredMessage` if one has been posted in the specified chat.
    ///
    /// Technical Note: This method actually creates a new `Channel` with your Urbit Ship, and spawns a new unix thread
    /// locally that processes all messages on said channel. This is required due to borrowing mechanisms in Rust, however
    /// on the plus side this makes it potentially more performant by each subscription having it's own unix thread.
    pub fn subscribe_to_chat(
        &mut self,
        chat_ship: &str,
        chat_name: &str,
    ) -> Result<Receiver<AuthoredMessage>> {
        let chat_ship = chat_ship.to_string();
        let chat_name = chat_name.to_string();
        // Create sender/receiver
        let (s, r) = unbounded();
        // Creating a new Ship Interface Channel to pass into the new thread
        // to be used to communicate with the Urbit ship
        let mut new_channel = self.channel.ship_interface.create_channel()?;

        thread::spawn(move || {
            // Infinitely watch for new graph store updates
            let channel = &mut new_channel;
            channel
                .create_new_subscription("graph-store", "/updates")
                .ok();
            loop {
                // Pause for half a second
                thread::sleep(Duration::new(0, 500000000));
                channel.parse_event_messages();
                let res_graph_updates = &mut channel.find_subscription("graph-store", "/updates");
                if let Some(graph_updates) = res_graph_updates {
                    // Read all of the current SSE messages to find if any are for the chat
                    // we are looking for.
                    loop {
                        let pop_res = graph_updates.pop_message();
                        // Acquire the message
                        if let Some(mess) = &pop_res {
                            // Parse it to json
                            if let Ok(json) = json::parse(mess) {
                                // If the graph-store node update is not for the correct chat
                                // then continue to next message.
                                if !Self::check_resource_json(&chat_ship, &chat_name, &json) {
                                    continue;
                                }
                                // Otherwise, parse json to a `Node`
                                if let Ok(node) = Node::from_graph_update_json(&json) {
                                    // Parse it as an `AuthoredMessage`
                                    let authored_message =
                                        AuthoredMessage::new(node.author, node.contents);
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
            }
        });
        Ok(r)
    }

    /// Checks whether the resource json matches the chat_name & chat_ship
    /// specified
    fn check_resource_json(chat_ship: &str, chat_name: &str, resource_json: &JsonValue) -> bool {
        let resource = resource_json["graph-update"]["add-nodes"]["resource"].clone();
        let chat_name = format!("{}", resource["name"]);
        let chat_ship = format!("~{}", resource["ship"]);
        if chat_name == chat_name && chat_ship == chat_ship {
            return true;
        }
        false
    }
}
