use std::{
    io::{self, Write},
    net::{SocketAddr, TcpStream},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use base64::{Engine as _, engine::general_purpose};
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::{crypto, message::Message, output::Output, util};

pub enum PeerStatus {
    /// We know about the peer but no crypto established yet.
    Disconnected { peer_id: String },

    /// We initiated DH and are waiting for a response.
    SentKeyExchange {
        peer_id: String,
        session_id: String,
        secret: EphemeralSecret,
    },

    /// We received init and generated our response.
    ReceivedKeyExchange {
        peer_id: String,
        session_id: String,
        aes_key: [u8; 32],
    },

    /// A secure channel has been established.
    Encrypted { peer_id: String, aes_key: [u8; 32] },
}

struct ClientData {
    id: Option<String>,
    peers: Vec<PeerStatus>,
}

pub struct Client {
    write: Arc<Mutex<TcpStream>>,
    reader: JoinHandle<()>,
    data: Arc<Mutex<ClientData>>,
    output: Arc<Mutex<Output>>,
}

fn handle_read(
    write: Arc<Mutex<TcpStream>>,
    data: Arc<Mutex<ClientData>>,
    output: Arc<Mutex<Output>>,
    msg: Message,
) {
    if let Message::AssignId { id } = &msg
        && let Ok(mut data) = data.lock()
    {
        data.id = Some(id.to_owned());
        return;
    }

    util::print(&output, format!("{msg:?}"));
}

impl Client {
    pub fn new(
        addr: SocketAddr,
        output: Arc<Mutex<Output>>,
    ) -> io::Result<Self> {
        let write = TcpStream::connect_timeout(&addr, Duration::from_secs(5))?;
        let read = write.try_clone()?;
        let write = Arc::new(Mutex::new(write));
        let data = Arc::new(Mutex::new(ClientData {
            id: None,
            peers: vec![],
        }));

        let write_clone = write.clone();
        let data_clone = data.clone();
        let output_clone = output.clone();

        let cb = move |msg| {
            handle_read(
                write_clone.clone(),
                data_clone.clone(),
                output_clone.clone(),
                msg,
            );
        };
        let reader =
            thread::spawn(move || util::read_from_stream(read, Box::new(cb)));

        Ok(Self {
            write,
            reader,
            data,
            output,
        })
    }

    pub fn send(&mut self, message: Message) {
        if let Ok(str) = serde_json::to_string(&message)
            && let Ok(mut write) = self.write.lock()
        {
            let _ = writeln!(write, "{str}");
        }
    }

    pub fn begin_key_exchange(&mut self, recipient_id: String) {
        let secret = EphemeralSecret::random();
        let public = PublicKey::from(&secret);

        let session_id = uuid::Uuid::new_v4().to_string();

        if let Ok(mut data) = self.data.lock() {
            data.peers.push(PeerStatus::SentKeyExchange {
                peer_id: recipient_id.clone(),
                session_id: session_id.clone(),
                secret,
            });
        }

        self.send(Message::KeyExchangeInit {
            sender_id: self.id().unwrap(),
            recipient_id,
            public_key: general_purpose::STANDARD.encode(public.as_bytes()),
        });
    }

    pub fn send_encrypted(&mut self, recipient_id: String, plaintext: String) {
        let key = {
            let data = self.data.lock().unwrap();

            data.peers.iter().find_map(|p| {
                if let PeerStatus::Encrypted { peer_id, aes_key } = p {
                    if peer_id == &recipient_id {
                        return Some(*aes_key);
                    }
                }

                None
            })
        };

        let Some(key) = key else {
            return;
        };

        let (nonce, ciphertext) = crypto::encrypt_message(&key, &plaintext);

        self.send(Message::Encrypted {
            sender_id: self.id().unwrap(),
            recipient_id,
            nonce,
            ciphertext,
        });
    }

    fn id(&self) -> Option<String> {
        self.data.lock().ok()?.id.clone()
    }
}
