# Rust Urbit HTTP API

This library wraps the Urbit ship http interface exposing it as an easy-to-use Rust crate.

All implementation details such as auth cookies, EventSource connections, tracking message ids, and other such matters are automatically handled for you, and as enables a greatly improved experience in writing Rust apps that interact with Urbit ships.

This crate currently enables devs to:

1. Authorize oneself and open a channel with the ship.
2. Subscribe to any app/path so that one can read the events currently taking place inside of the ship.
3. Issue `Poke`s to apps.
4. Send messages to an Urbit chat.
5. Issue generic Graph Store pokes.

## Basic Design

There are 3 main structs that this library exposes for interacting with an Urbit ship:

1. `ShipInterface`
2. `Channel`
3. `Subscription`

A `Subscription` is created by a `Channel` which is created by a `ShipInterface`. In other words, first you need to connect to an Urbit ship (using `ShipInterface`) before you can initiate a messaging `Channel`, before you can create a `Subscription` to an app/path.

### ShipInterface

The `ShipInterface` exposes a few useful methods that will be useful when creating apps.

The more commonly used methods below these allow you to create a new `ShipInterface` (thereby authorizing yourself with the ship), and create a new `Channel`.

```rust
/// Logs into the given ship and creates a new `ShipInterface`.
/// `ship_url` should be `http://ip:port` of the given ship. Example:
/// `http://0.0.0.0:8080`. `ship_code` is the code acquire from your ship
/// by typing `+code` in dojo.
pub fn new(ship_url: &str, ship_code: &str) -> Result<ShipInterface>;

/// Create a `Channel` using this `ShipInterface`
pub fn create_channel(&mut self) -> Result<Channel>;
```

You also have the ability to scry and run threads via spider.

```rust
/// Send a scry using the `ShipInterface`
pub fn scry(&self, app: &str, path: &str) -> Result<Response>;

/// Run a thread via spider using the `ShipInterface`
pub fn spider(&self, input_mark: &str, output_mark: &str, thread_name: &str, body: &JsonValue) -> Result<Response>;
```

### Channel

`Channel` is the most useful struct, because it holds methods related to interacting with both pokes and subscriptions.

It is instructive to look at the definition of the `Channel` struct to understand how it works:

```rust
// A Channel which is used to interact with a ship
pub struct Channel<'a> {
    /// `ShipInterface` this channel is created from
    pub ship_interface: &'a ShipInterface,
    /// The uid of the channel
    pub uid: String,
    /// The url of the channel
    pub url: String,
    // The list of `Subscription`s for this channel
    pub subscription_list: Vec<Subscription>,
    // / The `EventSource` for this channel which reads all of
    // / the SSE events.
    event_receiver: ReceiverSource,
    /// The current number of messages that have been sent out (which are
    /// also defined as message ids) via this `Channel`
    pub message_id_count: u64,
}
```

Once a `Channel` is created, an `EventSource` connection is created with the ship on a separate thread. This thread accepts all of the incoming events, and queues them on a (Rust) unbounded channel which is accessible internally via the `event_receiver`. This field itself isn't public, but processing events in this crate is handled with a much higher-level interface for the app developer.

Take note that a `Channel` has a `subscription_list`. As you will see below, each `Channel` exposes methods for creating subscriptions, which automatically get added to the `subscription_list`.
Once `Subscription`s are created/added to the list, the `Channel` will evidently start to receive event messages via SSE (which will be queued for reading in the `event_receiver`).

From the app developer's perspective, all one has to do is call the `parse_event_messages()` method on your `Channel`, and all of the queued events will be processed and passed on to the correct `Subscription`'s `message_list`. This is useful once multiple `Subscriptions` are created on a single channel, as the messages will be pre-sorted automatically for you.

Once the event messages are parsed, then one can simply call the `find_subscription` method in order to interact with the `Subscription` and read its messages.

The following are the useful methods exposed by a `Channel`:

```rust
/// Sends a poke over the channel
pub fn poke(&mut self, app: &str, mark: &str, json: &JsonValue) -> Result<Response>;

/// Create a new `Subscription` and thus subscribes to events on the ship with the provided app/path.
pub fn create_new_subscription(&mut self, app: &str, path: &str) -> Result<CreationID>;

/// Parses SSE messages for this channel and moves them into
/// the proper corresponding `Subscription`'s `message_list`.
pub fn parse_event_messages(&mut self);

/// Finds the first `Subscription` in the list which has a matching
/// `app` and `path`;
pub fn find_subscription(&self, app: &str, path: &str) -> Option<&Subscription>;

/// Finds the first `Subscription` in the list which has a matching
/// `app` and `path`, removes it from the list, and tells the ship
/// that you are unsubscribing.
pub fn unsubscribe(&mut self, app: &str, path: &str) -> Option<bool>;

/// Deletes the channel
pub fn delete_channel(self);

/// Exposes an interface for interacting with a ship's Graph Store directly.
pub fn graph_store(&mut self) -> GraphStore;

/// Exposes an interface for interacting with Urbit chats.
pub fn chat(&mut self) -> Chat;

/// Exposes an interface for interacting with Urbit notebooks.
pub fn notebook(&mute self) -> Notebook;

```

### Subscription

As mentioned in the previous section, a `Subscription` contains it's own `message_list` field where messages are stored after a `Channel` processes them.

From an app developer's perspective, this is the only useful feature of the `Subscription` struct. Once acquired, it is used simply to read the messages.

To improve the message reading experience, the `Subscription` struct exposes a useful method:

```rust
/// Pops a message from the front of `Subscription`'s `message_list`.
/// If no messages are left, returns `None`.
pub fn pop_message(&mut self) -> Option<String>;
```

## Code Examples

### Poke Example

This example displays how to connect to a ship using a `ShipInterface`, opening a `Channel`, issuing a `poke` over said channel, and then deleting the `Channel` to finish.

```rust
// Import the `ShipInterface` struct
use urbit_http_api::ShipInterface;

fn main() {
    // Create a new `ShipInterface` for a local ~zod ship
    let mut ship_interface =
        ShipInterface::new("http://0.0.0.0:8080", "lidlut-tabwed-pillex-ridrup").unwrap();
    // Create a `Channel`
    let mut channel = ship_interface.create_channel().unwrap();

    // Issue a poke over the channel
    let poke_res = channel.poke("hood", "helm-hi", &"This is a poke".into());

    // Cleanup/delete the `Channel` once finished
    channel.delete_channel();
}
```

### Graph Store Subscription Example

This example shows how to create, interact with, and delete a `Subscription`. In this scenario we desire to read all new updates from Graph Store via our `Subscription` for 10 seconds, and then perform cleanup.

```rust
use std::thread;
use std::time::Duration;
use urbit_http_api::ShipInterface;

fn main() {
    // Create a new `ShipInterface` for a local ~zod ship
    let mut ship_interface =
        ShipInterface::new("http://0.0.0.0:8080", "lidlut-tabwed-pillex-ridrup").unwrap();
    // Create a `Channel`
    let mut channel = ship_interface.create_channel().unwrap();
    // Create a `Subscription` for the `graph-store` app with the `/updates` path. This `Subscription`
    // is automatically added to the `Channel`'s `subscription_list`.
    channel
        .create_new_subscription("graph-store", "/updates")
        .unwrap();

    // Create a loop that iterates 10 times
    for _ in 0..10 {
        // Parse all of the event messages to move them into the correct
        // `Subscription`s in the `Channel`'s `subscription_list`.
        channel.parse_event_messages();

        // Find our graph-store `Subscription`
        let gs_sub = channel.find_subscription("graph-store", "/updates").unwrap();

        // Pop all of the messages from our `gs_sub` and print them
        loop {
            let pop_res = gs_sub.pop_message();
            if let Some(mess) = &pop_res {
                println!("Message: {:?}", mess);
            }
            // If no messages left, stop
            if let None = &pop_res {
                break;
            }
        }

        // Wait for 1 second before trying to parse the event messages again
        thread::sleep(Duration::new(1, 0));
    }

    // Once finished, unsubscribe/destroy our `Subscription`
    channel.unsubscribe("graph-store", "/updates");
    // Delete the channel
    channel.delete_channel();
}
```

### Urbit Chat Messaging Example

This example displays how to connect to a ship and send a message to an Urbit chat using the `Chat` struct interface.

```rust
// Import the `ShipInterface` struct
use urbit_http_api::{ShipInterface, chat::Message};

fn main() {
    // Create a new `ShipInterface` for a local ~zod ship
    let mut ship_interface =
        ShipInterface::new("http://0.0.0.0:8080", "lidlut-tabwed-pillex-ridrup").unwrap();
    // Create a `Channel`
    let mut channel = ship_interface.create_channel().unwrap();

    // Create a `Message` which is formatted properly for an Urbit chat
    let message = Message::new()
        // Add text to your message
        .add_text("Checkout this cool article by ~wicdev-wisryt:")
        // Add a URL link to your message after the previous text (which gets automatically added on a new line)
        .add_url("https://urbit.org/blog/io-in-hoon/")
        // Add an image URL to your message after the previous url (which gets automatically added on a new line as a rendered image)
        .add_url("https://media.urbit.org/site/posts/essays/zion-canyon-1.jpg");
    // Send the message to a chat hosted by ~zod named "test-93".
    // Note the connected ship must already have joined the chat in order to send a message to the chat.
    let _mess_res = channel
        .chat()
        .send_message("~zod", "test-93", &message);

    // Cleanup/delete the `Channel` once finished
    channel.delete_channel();
}
```

### Urbit Chat Subscription Example

This example shows how to utilize the higher-level `Chat` interface to subscribe to a chat and read all of the messages being posted in said chat.

```rust
use std::thread;
use std::time::Duration;
use urbit_http_api::ShipInterface;

fn main() {
    // Create a new `ShipInterface` for a local ~zod ship
    let mut ship_interface =
        ShipInterface::new("http://0.0.0.0:8080", "lidlut-tabwed-pillex-ridrup").unwrap();
    // Create a `Channel`
    let mut channel = ship_interface.create_channel().unwrap();
    // Subscribe to a specific chat, and obtain a `Receiver` back which contains a stream of messages from the chat
    let chat_receiver = channel
        .chat()
        .subscribe_to_chat("~mocrux-nomdep", "test-93")
        .unwrap();

    // Create a loop that iterates 10 times
    for _ in 0..10 {
        // If a message has been posted to the chat, unwrap it and acquire the `AuthoredMessage`
        if let Ok(authored_message) = chat_receiver.try_recv() {
            // Pretty print the author ship @p and the message contents
            println!(
                "~{}:{}",
                authored_message.author,
                authored_message.message.to_formatted_string()
            );
        }
        // Wait for 1 second before checking again
        thread::sleep(Duration::new(1, 0));
    }

    // Delete the channel
    channel.delete_channel();
}
```

---

This library was created by ~mocrux-nomdep([Robert Kornacki](https://github.com/robkorn)).
