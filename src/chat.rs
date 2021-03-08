use crate::error::Result;
use crate::traits::messaging::{AuthoredMessage, Message, Messaging};
use crate::{Channel, Node};

/// A struct that provides an interface for interacting with Urbit chats
pub struct Chat<'a> {
    pub channel: &'a mut Channel,
}

impl<'a> Messaging for Chat<'a> {
    fn channel(&mut self) -> &mut Channel {
        self.channel
    }
}

impl<'a> Chat<'a> {
    /// Send a message to an Urbit chat.
    /// Returns the index of the node that was added to Graph Store.
    pub fn send_chat_message(
        &mut self,
        chat_ship: &str,
        chat_name: &str,
        message: &Message,
    ) -> Result<String> {
        self.send_message(chat_ship, chat_name, message)
    }
}
