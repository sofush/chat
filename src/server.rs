use std::{
    io,
    net::{SocketAddr, TcpListener},
    ops::DerefMut,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use crate::{message::Message, output::Output, participant::Participant, util};
use openidconnect::AccessToken;
use owo_colors::OwoColorize as _;

pub fn authenticate_participant(
    output: Arc<Mutex<Output>>,
    access_token: &AccessToken,
) -> Option<String> {
    let token = access_token.secret();

    match crate::jwt::verify_jwt(token) {
        Ok(username) => Some(username),
        Err(e) => {
            util::error(&output, format!("Auth failed: {e}"));
            None
        }
    }
}

fn handle_participant_msg(
    id: String,
    message: Message,
    connections: Arc<Mutex<Vec<Participant>>>,
    output: Arc<Mutex<Output>>,
) {
    let Ok(mut connections) = connections.lock() else {
        return;
    };

    if let Message::Authenticate { access_token } = &message {
        let Some(username) =
            authenticate_participant(output.clone(), access_token)
        else {
            util::error(
                &output,
                "Authentication for a client failed.".red().to_string(),
            );
            return;
        };

        if let Some(newly_joined) =
            connections.iter_mut().find(|c| c.id() == id)
        {
            newly_joined.set_username(username.clone());
        }

        let id_username_pairs = connections
            .iter()
            .filter_map(|c| {
                let username = c.username().map(|s| s.to_owned())?;
                let id = c.id().to_owned();
                let pair = (id, username);
                Some(pair)
            })
            .collect::<Vec<_>>();

        if let Some(newly_joined) =
            connections.iter_mut().find(|c| c.id() == id)
        {
            for (peer_id, username) in id_username_pairs {
                let _ = newly_joined.send(Message::AnnounceUsername {
                    id: peer_id.to_string(),
                    username,
                });
            }
        }

        for participant in connections.deref_mut() {
            let _ = participant.send(Message::AnnounceJoin {
                id: id.to_string(),
                username: username.clone(),
            });
        }

        util::info(
            &output,
            format!("User {} has been authorized.", username.yellow().bold()),
        );
    }

    for c in &mut *connections {
        if c.id() == id {
            continue;
        }

        if c.send(message.clone()).is_err() {
            util::error(&output, "Failed to broadcast a message.");
        }
    }
}

#[allow(unused)]
pub struct Server {
    addr: SocketAddr,
    connections: Arc<Mutex<Vec<Participant>>>,
    connection_thread: Option<JoinHandle<()>>,
    reader: Option<JoinHandle<()>>,
    output: Arc<Mutex<Output>>,
}

impl Server {
    pub fn new(
        addr: SocketAddr,
        output: Arc<Mutex<Output>>,
    ) -> io::Result<Self> {
        let connections: Arc<Mutex<Vec<Participant>>> = Default::default();

        Ok(Self {
            connections,
            connection_thread: None,
            reader: None,
            addr,
            output,
        })
    }

    pub fn host(&mut self) -> io::Result<()> {
        let listener = TcpListener::bind(self.addr)?;
        let connections = self.connections.clone();

        let broadcast_fn_connections = connections.clone();
        let output_clone = self.output.clone();
        let broadcast_fn = move |id: String, msg: Message| {
            handle_participant_msg(
                id,
                msg,
                broadcast_fn_connections.clone(),
                output_clone.clone(),
            );
        };

        let output_clone = self.output.clone();
        self.connection_thread = Some(thread::spawn(move || {
            loop {
                let func = broadcast_fn.clone();

                let Ok((stream, _)) = listener.accept() else {
                    break;
                };

                let mut c = connections.lock().unwrap();
                let uuid = uuid::Uuid::new_v4().to_string();
                let uuid_clone = uuid.clone();
                let boxed = Box::new(move |msg: Message| {
                    func(uuid_clone.clone(), msg.clone())
                });

                if let Ok(participant) =
                    Participant::new(uuid.clone(), stream, boxed)
                {
                    util::debug(
                        &output_clone.clone(),
                        format!("New client: {}", uuid.yellow()),
                    );

                    c.push(participant);
                }
            }
        }));

        Ok(())
    }
}
