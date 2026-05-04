use std::{
    io::{self, Write},
    net::{SocketAddr, TcpStream},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{message::Message, util};
use uuid::Uuid;

pub struct Client {
    write: TcpStream,
    reader: JoinHandle<()>,
    uuid: Uuid,
}

impl Client {
    pub fn new(addr: SocketAddr) -> io::Result<Self> {
        let write = TcpStream::connect_timeout(&addr, Duration::from_secs(5))?;
        let read = write.try_clone()?;
        let cb = |msg| {
            println!("{msg:?}");
        };
        let reader =
            thread::spawn(move || util::read_from_stream(read, Box::new(cb)));

        let uuid = uuid::Uuid::new_v4();
        let mut this = Self {
            write,
            reader,
            uuid,
        };
        this.send(Message::Join(uuid.to_string()));
        Ok(this)
    }

    pub fn send(&mut self, message: Message) {
        if let Ok(str) = serde_json::to_string(&message) {
            let _ = writeln!(self.write, "{str}");
        }
    }
}
