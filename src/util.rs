use std::{io::BufReader, net::TcpStream};

use serde::Deserialize as _;

use crate::message::Message;

pub fn read_from_stream(stream: TcpStream, callback: Box<dyn Fn(Message)>) {
    let mut reader = BufReader::new(stream);

    loop {
        let mut de = serde_json::Deserializer::from_reader(&mut reader);

        let Ok(message) = Message::deserialize(&mut de) else {
            return;
        };

        callback(message)
    }
}
