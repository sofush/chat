use std::{
    io::{self, Write as _},
    net::TcpStream,
    thread::{self, JoinHandle},
};

use crate::{message::Message, util};

#[allow(unused)]
pub struct Participant {
    write: TcpStream,
    id: String,
    reader: JoinHandle<()>,
    username: Option<String>,
}

impl Participant {
    pub fn new(
        id: String,
        stream: TcpStream,
        callback: Box<dyn Fn(Message) + Send>,
    ) -> io::Result<Self> {
        let clone = stream.try_clone()?;

        let reader = thread::spawn(move || {
            util::read_from_stream(clone, callback);
        });

        let mut this = Self {
            id: id.clone(),
            write: stream,
            reader,
            username: None,
        };

        this.send(Message::AssignId { id: id.to_string() })?;
        Ok(this)
    }

    pub fn send(&mut self, msg: Message) -> serde_json::error::Result<()> {
        let serialized = serde_json::to_string(&msg)?;
        let _ = writeln!(self.write, "{serialized}");
        Ok(())
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn username(&self) -> Option<&str> {
        self.username.as_deref()
    }

    pub fn set_username(&mut self, username: String) {
        self.username = Some(username);
    }
}
