use crate::error::Result;
use crate::traits::messaging::{AuthoredMessage, Message, Messaging};
use crate::Channel;
use crossbeam::channel::Receiver;

/// A struct that provides an interface for interacting with Urbit DMs
pub struct DM<'a> {
    pub channel: &'a mut Channel,
}

impl<'a> Messaging for DM<'a> {
    fn channel(&mut self) -> &mut Channel {
        self.channel
    }
}

impl<'a> DM<'a> {
    /// Converts a ship @p to the `dm_name` string format used for DM channels
    pub fn ship_to_dm_name(&self, ship: &str) -> String {
        format!("dm--{}", ship)
    }

    /// Send a message to an Urbit DM chat.
    /// Returns the index of the node that was added to Graph Store.
    pub fn send_dm_message(
        &mut self,
        dm_ship: &str,
        dm_name: &str,
        message: &Message,
    ) -> Result<String> {
        self.send_message(dm_ship, dm_name, message)
    }

    /// Extracts DM chat log automatically into a list of formatted `String`s
    pub fn export_dm_log(&mut self, dm_ship: &str, dm_name: &str) -> Result<Vec<String>> {
        self.export_message_log(dm_ship, dm_name)
    }

    /// Extracts a DM chat's messages as `AuthoredMessage`s
    pub fn export_dm_authored_messages(
        &mut self,
        dm_ship: &str,
        dm_name: &str,
    ) -> Result<Vec<AuthoredMessage>> {
        self.export_authored_messages(dm_ship, dm_name)
    }

    /// Subscribe to and watch for messages. This method returns a `Receiver` with the
    /// `AuthoredMessage`s that are posted after subscribing. Simply call `receiver.try_recv()`
    /// to read the next `AuthoredMessage` if one has been posted.
    ///
    /// Technical Note: This method actually creates a new `Channel` with your Urbit Ship, and spawns a new unix thread
    /// locally that processes all messages on said channel. This is required due to borrowing mechanisms in Rust, however
    /// on the plus side this makes it potentially more performant by each subscription having it's own unix thread.
    pub fn subscribe_to_dm(
        &mut self,
        dm_ship: &str,
        dm_name: &str,
    ) -> Result<Receiver<AuthoredMessage>> {
        self.subscribe_to_messages(dm_ship, dm_name)
    }
}
