use crate::chat::AuthoredMessage;

/// A struct representing a comment either on a `Note` or on a collections `Link`.
/// Matches the `AuthoredMessage` struct, and as such is a type alias for it.
pub type Comment = AuthoredMessage;
