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
    #[error("Failed to fetch Graph Store keys.")]
    FailedToFetchKeys,
    #[error("Failed to fetch Graph Store tags.")]
    FailedToFetchTags,
    #[error("Failed to send a chat message to chat {0}.")]
    FailedToSendChatMessage(String),
    #[error("Failed to acquire update log from Graph Store for resource {0}.")]
    FailedToGetUpdateLog(String),
    #[error("Failed to acquire graph from Graph Store for resource {0}.")]
    FailedToGetGraph(String),
    #[error("Failed to acquire graph node from Graph Store for resource + index {0}.")]
    FailedToGetGraphNode(String),
    #[error("Failed to archive graph from Graph Store for resource {0}.")]
    FailedToArchiveGraph(String),
    #[error("Failed to add tag to resource {0}.")]
    FailedToAddTag(String),
    #[error("Failed to remove tag from resource {0}.")]
    FailedToRemoveTag(String),
    #[error("Failed to add nodes to Graph Store for resource {0}.")]
    FailedToAddNodesToGraphStore(String),
    #[error("Failed to remove nodes from Graph Store for resource {0}.")]
    FailedToRemoveNodesFromGraphStore(String),
    #[error("Failed to remove graph from Graph Store for resource {0}.")]
    FailedToRemoveGraphFromGraphStore(String),
    #[error("Failed to build a Graph struct from supplied JsonValue.")]
    FailedToCreateGraphFromJSON,
    #[error("Failed to build a Node struct from supplied JsonValue.")]
    FailedToCreateGraphNodeFromJSON,
    #[error("Failed to insert a Node struct into a Graph because of the index.")]
    FailedToInsertGraphNode,
    #[error("The following graph node is not a valid Notebook Note node {0}")]
    InvalidNoteGraphNode(String),
    #[error("The following graph node is not a valid Collections Link node {0}")]
    InvalidLinkGraphNode(String),
    #[error("The following graph node index is not a valid Notebook Note node index {0}")]
    InvalidNoteGraphNodeIndex(String),
    #[error("Failed to create a Notebook Note from these nodes {0}")]
    FailedToCreateNote(String),
    #[error("Failed to create a Notebook Comment from these nodes {0}")]
    FailedToCreateComment(String),
    #[error("The following graph node index is not a valid Notebook Comment node index {0}")]
    InvalidCommentGraphNodeIndex(String),
    #[error("{0}")]
    Other(String),
    #[error(transparent)]
    ReqwestError(#[from] ReqError),
}
