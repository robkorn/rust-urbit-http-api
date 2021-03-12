use crate::{Channel, Node, Result, UrbitAPIError};
use crossbeam::channel::{unbounded, Receiver};
use json::JsonValue;
use std::thread;
use std::time::Duration;

/// A struct that provides an interface for interacting with invite-store
pub struct InviteStore<'a> {
    pub channel: &'a mut Channel,
}

impl<'a> InviteStore<'a> {
    /// Accept an invite
    pub fn accept_invite(&self, term: &str, uid: &str) {
        // let mut poke2_data = json::JsonValue::new_object();
        // poke2_data["accept"] = json::JsonValue::new_object();
        // poke2_data["accept"]["term"] = "graph".to_string().into();
        // poke2_data["accept"]["uid"] = poke_channel.uid.clone().into();
        // let _poke2_response = poke_channel.poke("invite-store", "invite-action", &poke_data);
        todo!();
    }
}
