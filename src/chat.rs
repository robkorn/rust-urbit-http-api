use crate::error::Result;
use crate::traits::messaging::{AuthoredMessage, Message, Messaging};
use crate::Channel;
use crossbeam::channel::Receiver;

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

    /// Extracts chat log automatically into a list of formatted `String`s
    pub fn export_chat_log(&mut self, chat_ship: &str, chat_name: &str) -> Result<Vec<String>> {
        self.export_message_log(chat_ship, chat_name)
    }

    pub fn export_chat_authored_messages(
        &mut self,
        chat_ship: &str,
        chat_name: &str,
    ) -> Result<Vec<AuthoredMessage>> {
        self.export_authored_messages(chat_ship, chat_name)
    }

    /// Subscribe to and watch for messages. This method returns a `Receiver` with the
    /// `AuthoredMessage`s that are posted after subscribing. Simply call `receiver.try_recv()`
    /// to read the next `AuthoredMessage` if one has been posted.
    ///
    /// Technical Note: This method actually creates a new `Channel` with your Urbit Ship, and spawns a new unix thread
    /// locally that processes all messages on said channel. This is required due to borrowing mechanisms in Rust, however
    /// on the plus side this makes it potentially more performant by each subscription having it's own unix thread.
    pub fn subscribe_to_chat(
        &mut self,
        chat_ship: &str,
        chat_name: &str,
    ) -> Result<Receiver<AuthoredMessage>> {
        self.subscribe_to_messages(chat_ship, chat_name)
    }
}
