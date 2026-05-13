use std::{
    io,
    net::{SocketAddr, TcpListener},
    ops::DerefMut,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use owo_colors::OwoColorize as _;

use crate::{message::Message, output::Output, participant::Participant, util};

fn authorize_message(message: &Message) -> bool {
    true
}

fn handle_participant_msg(
    message: Message,
    connections: Arc<Mutex<Vec<Participant>>>,
    output: Arc<Mutex<Output>>,
) {
    let Ok(mut connections) = connections.lock() else {
        return;
    };

    if !authorize_message(&message) {
        return;
    }

    for c in &mut *connections {
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
        let broadcast_fn = move |msg: Message| {
            handle_participant_msg(
                msg,
                broadcast_fn_connections.clone(),
                output_clone.clone(),
            );
        };

        let output_clone = self.output.clone();
        self.connection_thread = Some(thread::spawn(move || {
            loop {
                if let Ok((stream, _)) = listener.accept() {
                    let mut c = connections.lock().unwrap();
                    let uuid = uuid::Uuid::new_v4();

                    if let Ok(participant) = Participant::new(
                        uuid,
                        stream,
                        Box::new(broadcast_fn.clone()),
                    ) {
                        util::debug(
                            &output_clone.clone(),
                            format!("New client: {}", uuid.yellow()),
                        );

                        for participant in c.deref_mut() {
                            let _ = participant.send(Message::AnnounceJoin {
                                id: uuid.to_string(),
                            });
                        }

                        c.push(participant);
                    }
                }
            }
        }));

        Ok(())
    }
}
