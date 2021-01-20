use crate::{Channel, Result};
use json::{object, JsonValue};

pub struct Chat<'a> {
    pub channel: &'a mut Channel,
}

impl<'a> Chat<'a> {
    pub fn send_message(
        &mut self,
        chat_ship: &str,
        chat_name: &str,
        message: &JsonValue,
    ) -> Result<()> {
        let contents = vec![object! {
            "text": message.clone()
        }];

        self.channel
            .graph_store()
            .post(chat_ship, chat_name, contents)
    }
}
