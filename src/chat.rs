use crate::{Channel, Result};
use json::{object, JsonValue};

/// A struct that provides an interface for interacting with Urbit chats
pub struct Chat<'a> {
    pub channel: &'a mut Channel,
}

/// A struct that represents a message that is to be sent to an Urbit chat.
/// `Message` provides methods to build a message in chunks, thereby allowing you
/// to add content which needs to be parsed, for example links or code.
#[derive(Debug, Clone)]
pub struct Message {
    contents: Vec<JsonValue>,
}

/// Methods for interacting with a Chat
impl<'a> Chat<'a> {
    /// Send a message to an Urbit chat
    pub fn send_message(
        &mut self,
        chat_ship: &str,
        chat_name: &str,
        message: &Message,
    ) -> Result<()> {
        self.channel
            .graph_store()
            .post(chat_ship, chat_name, message.clone().contents)
    }
}

/// Methods for creating/building a `Message`
impl Message {
    /// Create a new empty `Message`
    pub fn new() -> Message {
        Message { contents: vec![] }
    }

    /// Appends text to the end of the current message.
    pub fn add_text(&self, text: &str) -> Message {
        let formatted = object! {
            "text": text
        };
        self.add_to_message(formatted)
    }

    /// Appends a url to the end of the current message.
    pub fn add_url(&self, url: &str) -> Message {
        let formatted = object! {
            "url": url
        };
        self.add_to_message(formatted)
    }

    // These two need to be implemented properly in the coming future.
    // For future reference, post.hoon: https://github.com/urbit/urbit/blob/master/pkg/arvo/sur/post.hoon
    //
    // /// Appends a code block to the end of the current message.
    // pub fn add_code(&self, code: &str) -> Message {
    //     let formatted = object! {
    //         "code": code
    //     };
    //     self.add_to_message(formatted)
    // }

    // /// Appends a reference to another @p/ship to the end of the current message.
    // pub fn add_reference(&self, referenced_ship: &str) -> Message {
    //     let formatted = object! {
    //         "reference": referenced_ship
    //     };
    //     self.add_to_message(formatted)
    // }

    /// Internal method to append JsonValue to message
    fn add_to_message(&self, json: JsonValue) -> Message {
        let mut contents = self.contents.clone();
        contents.append(&mut vec![json]);
        Message { contents: contents }
    }
}
