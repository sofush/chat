use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Message {
    Internal(),
    /// A message sent by a user.
    User(String),
}
