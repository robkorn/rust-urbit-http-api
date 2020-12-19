use eventsource_threaded::event::Event;
use json;

// ID of the message that created a `Subscription`
pub type CreationID = u64;

// A subscription on a given Channel
#[derive(Debug, Clone)]
pub struct Subscription {
    /// The uid of the channel this subscription was made in
    pub channel_uid: String,
    /// The id of the message that created this subscription
    pub creation_id: CreationID,
    /// The app that is being subscribed to
    pub app: String,
    /// The path of the app being subscribed to
    pub path: String,
    // A list of messages from the given subscription.
    pub message_list: Vec<String>,
}

impl Subscription {
    /// Verifies if the id of the message id in the event matches thea
    /// `Subscription` `creation_id`.
    fn event_matches(&self, event: &Event) -> bool {
        if let Some(json) = &json::parse(&event.data).ok() {
            return self.creation_id.to_string() == json["id"].dump();
        }
        false
    }

    /// Parses an event and adds it to the message list if it's id
    /// matches the `Subscription` `creation_id`. On success returns
    /// the length of the message list.
    pub fn add_to_message_list(&mut self, event: &Event) -> Option<u64> {
        if self.event_matches(&event) {
            let json = &json::parse(&event.data).ok()?["json"];
            if !json.is_null() {
                self.message_list.push(json.dump());
                return Some(self.message_list.len() as u64);
            }
        }
        None
    }

    /// Pops a message from the front of `Subscription`'s `message_list`.
    /// If no messages are left, returns `None`.
    pub fn pop_message(&mut self) -> Option<String> {
        if self.message_list.len() == 0 {
            return None;
        }
        let messages = self.message_list.clone();
        let (head, tail) = messages.split_at(1);
        self.message_list = tail.to_vec();
        Some(head.to_owned()[0].clone())
    }
}
