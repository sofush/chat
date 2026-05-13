use openidconnect::AccessToken;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Message {
    Authenticate {
        access_token: AccessToken,
    },

    /// Announcement that a user has connected to the chat room.
    AnnounceJoin {
        id: String,
    },

    /// A text message encrypted with the established shared AES key.
    Encrypted {
        sender_id: String,
        recipient_id: String,

        /// AES-GCM ciphertext encoded as base64/hex.
        ciphertext: String,

        /// Nonce/IV used for AES encryption.
        nonce: String,
    },

    /// A message sent from server to client, informing the client of their ID.
    AssignId {
        id: String,
    },

    /// Initiates a Diffie-Hellman key exchange.
    KeyExchange {
        sender_id: String,
        recipient_id: String,

        /// Sender's ephemeral DH public key.
        public_key: String,
    },
}
