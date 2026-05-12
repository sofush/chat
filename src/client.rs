use std::{
    collections::HashMap,
    io::{self, Write},
    net::{SocketAddr, TcpStream},
    ops::DerefMut,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use base64::{Engine as _, engine::general_purpose};
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::{crypto, message::Message, output::Output, util};

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
    ) -> io::Result<Self> {
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

        Ok(Self {
            write,
            reader,
            data,
            output,
        })
    }

    pub fn send(&mut self, msg: Message) -> std::io::Result<()> {
        if let Ok(mut write) = self.write.lock()
            && let Ok(s) = serde_json::to_string(&msg)
        {
            writeln!(write, "{s}")?;
        }

        Ok(())
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

    util::print(&output, msg.clone());

    match msg {
        Message::AssignId { id } => {
            data.id = Some(id);
        }
        Message::AnnounceJoin { id: peer_id } => {
            do_key_exchange(data.deref_mut(), write.deref_mut(), peer_id, None)
        }
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
            );
        }
        Message::Encrypted {
            sender_id,
            recipient_id,
            ciphertext,
            nonce,
        } => todo!(),
        _ => (),
    }
}

fn do_key_exchange(
    data: &mut ClientData,
    write: &mut TcpStream,
    peer_id: String,
    peer_public_key: Option<PublicKey>,
) {
    let Some(my_id) = data.id.clone() else {
        return;
    };

    let status = data.get_status(&peer_id);

    if let Some(ppk) = peer_public_key
        && let Some(secret) = status.my_secret.take()
    {
        let aes_key = crypto::derive_aes_key(secret, &ppk);
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

// pub fn send_encrypted(&mut self, recipient_id: String, plaintext: String) {
//     let key = {
//         let data = self.data.lock().unwrap();
//
//         data.peers.iter().find_map(|p| {
//             if let PeerStatus::Encrypted { peer_id, aes_key } = p {
//                 if peer_id == &recipient_id {
//                     return Some(*aes_key);
//                 }
//             }
//
//             None
//         })
//     };
//
//     let Some(key) = key else {
//         return;
//     };
//
//     let (nonce, ciphertext) = crypto::encrypt_message(&key, &plaintext);
//
//     self.send(Message::Encrypted {
//         sender_id: self.id().unwrap(),
//         recipient_id,
//         nonce,
//         ciphertext,
//     });
// }
