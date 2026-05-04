use std::{
    io::{self, Write},
    net::{SocketAddr, TcpStream},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{message::Message, util};

pub struct Client {
    write: TcpStream,
    reader: JoinHandle<()>,
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

        Ok(Self { write, reader })
    }

    pub fn send(&mut self, message: Message) {
        if let Ok(str) = serde_json::to_string(&message) {
            let _ = writeln!(self.write, "{str}");
        }
    }
}
