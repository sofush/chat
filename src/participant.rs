use std::{
    io::{self, Write as _},
    net::TcpStream,
    thread::{self, JoinHandle},
};

use crate::{message::Message, util};

pub struct Participant {
    write: TcpStream,
    reader: JoinHandle<()>,
}

impl Participant {
    pub fn new(
        stream: TcpStream,
        callback: Box<dyn Fn(Message) + Send>,
    ) -> io::Result<Self> {
        let clone = stream.try_clone()?;

        let reader = thread::spawn(move || {
            util::read_from_stream(clone, callback);
        });

        Ok(Self {
            write: stream,
            reader: reader,
        })
    }

    pub fn send(&mut self, msg: Message) -> serde_json::error::Result<()> {
        let serialized = serde_json::to_string(&msg)?;
        let _ = writeln!(self.write, "{serialized}");
        Ok(())
    }
}
