use std::{
    collections::HashMap,
    io::Write,
    net::{SocketAddr, TcpStream},
    ops::DerefMut,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use base64::{Engine as _, engine::general_purpose};
use crossterm::style::Stylize;
use openidconnect::AccessToken;
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::{
    crypto,
    message::Message,
    openid,
    output::{Label, Output},
    util,
};

pub struct PeerStatus {
    has_sent_public_key: bool,
    my_secret: Option<EphemeralSecret>,
    my_public_key: PublicKey,
    aes_key: Option<[u8; 32]>,
}

struct ClientData {
    id: Option<String>,
    peers: HashMap<String, PeerStatus>,
}

#[allow(unused)]
pub struct Client {
    write: Arc<Mutex<TcpStream>>,
    reader: JoinHandle<()>,
    data: Arc<Mutex<ClientData>>,
    output: Arc<Mutex<Output>>,
}

impl Client {
    pub fn new(
        addr: SocketAddr,
        output: Arc<Mutex<Output>>,
    ) -> anyhow::Result<Self> {
        let write = TcpStream::connect_timeout(&addr, Duration::from_secs(5))?;
        let read = write.try_clone()?;
        let write = Arc::new(Mutex::new(write));
        let data = Arc::new(Mutex::new(ClientData {
            id: None,
            peers: HashMap::new(),
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

        let mut this = Self {
            write,
            reader,
            data,
            output,
        };

        this.authenticate()?;
        Ok(this)
    }

    pub fn authenticate(&mut self) -> anyhow::Result<()> {
        let mut write = self
            .write
            .lock()
            .map_err(|_| anyhow::anyhow!("Could not lock write."))?;
        let access_token = openid::authorize(self.output.clone())?;
        let authenticate_msg = Message::Authenticate { access_token };
        writeln!(write, "{}", serde_json::to_string(&authenticate_msg)?)?;
        Ok(())
    }

    pub fn send(&mut self, plaintext: String) {
        let Ok(mut data) = self.data.lock() else {
            return;
        };

        let Ok(mut write) = self.write.lock() else {
            return;
        };

        let keys = data.peers.keys().cloned().collect::<Vec<_>>();

        for peer_id in keys {
            send_encrypted(
                data.deref_mut(),
                write.deref_mut(),
                peer_id.to_string(),
                plaintext.clone(),
            );
        }
    }
}

impl ClientData {
    pub fn get_status(&mut self, peer_id: &str) -> &mut PeerStatus {
        self.peers.entry(peer_id.to_string()).or_insert_with(|| {
            let secret = EphemeralSecret::random();
            let public_key = PublicKey::from(&secret);

            PeerStatus {
                has_sent_public_key: false,
                my_secret: Some(secret),
                my_public_key: public_key,
                aes_key: None,
            }
        })
    }
}

fn handle_read(
    write: Arc<Mutex<TcpStream>>,
    data: Arc<Mutex<ClientData>>,
    output: Arc<Mutex<Output>>,
    msg: Message,
) {
    let Ok(mut data) = data.lock() else {
        return;
    };

    let Ok(mut write) = write.lock() else {
        return;
    };

    util::debug(&output, msg.clone());

    match msg {
        Message::AssignId { id } => {
            data.id = Some(id);
        }
        Message::AnnounceJoin { id: peer_id } => do_key_exchange(
            data.deref_mut(),
            write.deref_mut(),
            peer_id,
            None,
            output.clone(),
        ),
        Message::KeyExchange {
            sender_id: peer_id,
            recipient_id,
            public_key,
        } => {
            let Some(id) = &data.id else {
                return;
            };

            if *id != recipient_id {
                return;
            }

            let Some(public_key) =
                crypto::get_public_key_from_bytes(public_key.as_bytes())
            else {
                return;
            };

            do_key_exchange(
                data.deref_mut(),
                write.deref_mut(),
                peer_id,
                Some(public_key),
                output.clone(),
            );
        }
        Message::Encrypted {
            sender_id,
            recipient_id,
            ciphertext,
            nonce,
        } => {
            let Some(id) = data.id.as_ref() else {
                return;
            };

            if recipient_id != *id {
                return;
            };

            recv_encrypted(
                data.deref_mut(),
                sender_id,
                ciphertext,
                nonce,
                output,
            )
        }
        _ => (),
    }
}

fn do_key_exchange(
    data: &mut ClientData,
    write: &mut TcpStream,
    peer_id: String,
    peer_public_key: Option<PublicKey>,
    output: Arc<Mutex<Output>>,
) {
    let Some(my_id) = data.id.clone() else {
        return;
    };

    let status = data.get_status(&peer_id);

    if let Some(ppk) = peer_public_key
        && let Some(secret) = status.my_secret.take()
    {
        let aes_key = crypto::derive_aes_key(secret, &ppk);
        let b64_aes_key = general_purpose::STANDARD.encode(aes_key);
        util::debug(
            &output,
            format!("Derived AES key: {}", b64_aes_key.yellow().bold()),
        );
        status.aes_key = Some(aes_key);
    }

    if !status.has_sent_public_key {
        let msg = Message::KeyExchange {
            sender_id: my_id,
            recipient_id: peer_id.clone(),
            public_key: general_purpose::STANDARD.encode(status.my_public_key),
        };

        if let Ok(s) = serde_json::to_string(&msg)
            && writeln!(write, "{s}").is_ok()
        {
            status.has_sent_public_key = true;
        }
    }
}

fn send_encrypted(
    data: &mut ClientData,
    write: &mut TcpStream,
    peer_id: String,
    plaintext: String,
) {
    let Some(my_id) = data.id.clone() else {
        return;
    };

    let Some(aes_key) = data.get_status(&peer_id).aes_key else {
        return;
    };

    let (nonce, ciphertext) = crypto::encrypt_message(&aes_key, &plaintext);

    let msg = Message::Encrypted {
        sender_id: my_id,
        recipient_id: peer_id,
        nonce,
        ciphertext,
    };

    let serialized_message = serde_json::to_string(&msg).unwrap();
    let _ = writeln!(write, "{serialized_message}");
}

fn recv_encrypted(
    data: &mut ClientData,
    peer_id: String,
    ciphertext: String,
    nonce: String,
    output: Arc<Mutex<Output>>,
) {
    let Some(aes_key) = data.get_status(&peer_id).aes_key else {
        return;
    };

    if let Some(plaintext) =
        crypto::decrypt_message(&aes_key, &nonce, &ciphertext)
    {
        util::print(
            &output,
            Label::Info,
            format!("{} {plaintext}", (peer_id + ":").yellow().bold()),
        );
    };
}
