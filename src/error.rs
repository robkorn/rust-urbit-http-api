use reqwest::Error as ReqError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, UrbitAPIError>;

#[derive(Error, Debug)]
pub enum UrbitAPIError {
    #[error("Failed logging in to the ship given the provided url and code.")]
    FailedToLogin,
    #[error("Failed to create a new channel.")]
    FailedToCreateNewChannel,
    #[error("Failed to create a new subscription.")]
    FailedToCreateNewSubscription,
    #[error("Failed to send a chat message to chat {0}.")]
    FailedToSendChatMessage(String),
    #[error("{0}")]
    Other(String),
    #[error(transparent)]
    ReqwestError(#[from] ReqError),
}
