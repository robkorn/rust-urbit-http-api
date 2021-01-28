use crate::graph::NodeContents;
use crate::{Channel, Result, UrbitAPIError};

/// A struct that provides an interface for interacting with Urbit chats
pub struct Chat<'a> {
    pub channel: &'a mut Channel,
}

/// A struct that represents a message that is to be sent to an Urbit chat.
/// `Message` provides methods to build a message in chunks, thereby allowing you
/// to add content which needs to be parsed, for example links @p mentions.
/// It is technically an alias for the `NodeContents` struct.
pub type Message = NodeContents;

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

        for node in chat_graph.clone().nodes {
            export_log.push(node.to_formatted_string());
        }

        Ok(export_log)
    }
}
