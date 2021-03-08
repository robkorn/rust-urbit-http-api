use crate::{Channel, Node, Result, UrbitAPIError};
use crossbeam::channel::{unbounded, Receiver};
use json::JsonValue;
use std::thread;
use std::time::Duration;

/// A struct that provides an interface for interacting with hark-store
pub struct HarkStore<'a> {
    pub channel: &'a mut Channel,
}

impl<'a> HarkStore<'a> {}
