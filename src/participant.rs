use std::{
    io::{self, Write as _},
    net::TcpStream,
    thread::{self, JoinHandle},
};

use crate::{message::Message, util};
use uuid::Uuid;

pub struct Participant {
    write: TcpStream,
    id: Uuid,
    reader: JoinHandle<()>,
}

impl Participant {
    pub fn new(
        id: Uuid,
        stream: TcpStream,
        callback: Box<dyn Fn(Message) + Send>,
    ) -> io::Result<Self> {
        let clone = stream.try_clone()?;

        let reader = thread::spawn(move || {
            util::read_from_stream(clone, callback);
        });

        let mut this = Self {
            id,
            write: stream,
            reader,
        };

        this.send(Message::AssignId { id: id.to_string() })?;
        Ok(this)
    }

    pub fn send(&mut self, msg: Message) -> serde_json::error::Result<()> {
        let serialized = serde_json::to_string(&msg)?;
        let _ = writeln!(self.write, "{serialized}");
        Ok(())
    }

    pub fn id(&self) -> String {
        self.id.to_string()
    }
}
