use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Message {
    Internal(),
    /// A message containing a UUID of the user joining.
    Join(String),
    /// A text message sent by a user.
    User(String),
}
