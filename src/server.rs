use std::{
    io,
    net::{SocketAddr, TcpListener},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use crate::{message::Message, participant::Participant};

fn do_broadcast(message: Message, connections: Arc<Mutex<Vec<Participant>>>) {
    let Ok(mut connections) = connections.lock() else {
        return;
    };

    for c in &mut *connections {
        if c.send(message.clone()).is_err() {
            println!("Failed to broadcast a message.");
        }
    }
}

pub struct Server {
    addr: SocketAddr,
    connections: Arc<Mutex<Vec<Participant>>>,
    connection_thread: Option<JoinHandle<()>>,
    reader: Option<JoinHandle<()>>,
}

impl Server {
    pub fn new(addr: SocketAddr) -> io::Result<Self> {
        let connections: Arc<Mutex<Vec<Participant>>> = Default::default();

        Ok(Self {
            connections,
            connection_thread: None,
            reader: None,
            addr,
        })
    }

    pub fn host(&mut self) -> io::Result<()> {
        let listener = TcpListener::bind(self.addr)?;
        let connections = self.connections.clone();

        let broadcast_fn_connections = connections.clone();
        let broadcast_fn = move |msg: Message| {
            do_broadcast(msg, broadcast_fn_connections.clone());
        };

        self.connection_thread = Some(thread::spawn(move || {
            loop {
                if let Ok((stream, _)) = listener.accept() {
                    let mut c = connections.lock().unwrap();

                    if let Ok(participant) =
                        Participant::new(stream, Box::new(broadcast_fn.clone()))
                    {
                        println!("Client connected.");
                        c.push(participant);
                    }
                }
            }
        }));

        Ok(())
    }
}
