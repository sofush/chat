use std::{
    io::{self, BufReader, Read, Write as _},
    net::TcpStream,
    thread::{self, JoinHandle},
};

use serde::Deserialize;

use crate::message::Message;

pub struct Participant {
    write: TcpStream,
    reader: JoinHandle<()>,
}

pub fn read_from_stream(
    stream: TcpStream,
    callback: Box<dyn Fn(Message) -> ()>,
) {
    let mut reader = BufReader::new(stream);

    loop {
        let mut de = serde_json::Deserializer::from_reader(&mut reader);
        let Ok(message) = Message::deserialize(&mut de) else {
            return;
        };

        callback(message)
    }
}

impl Participant {
    pub fn new(
        stream: TcpStream,
        callback: Box<dyn Fn(Message) -> () + Send>,
    ) -> io::Result<Self> {
        let clone = stream.try_clone()?;

        let reader = thread::spawn(move || {
            read_from_stream(clone, callback);
        });

        Ok(Self {
            write: stream,
            reader: reader,
        })
    }

    pub fn send(&mut self, _message: Message) {
        let _ = writeln!(self.write, "Hello, world!");
    }
}
