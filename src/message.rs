use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Message {
    /// A text message sent by a user.
    User(String),
    /// A message sent from server to client, informing the client of their ID.
    AssignId(String),
}
