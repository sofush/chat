use std::{
    io::BufReader,
    net::TcpStream,
    sync::{Arc, Mutex},
};

use serde::Deserialize as _;

use crate::{
    message::Message,
    output::{Output, Printable},
};

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

pub fn print(output: &Arc<Mutex<Output>>, message: impl Into<Printable>) {
    if let Ok(output) = output.lock() {
        output.print(message);
    }
}

pub fn clear_input(output: &Arc<Mutex<Output>>) {
    if let Ok(output) = output.lock() {
        output.clear_input();
    }
}

pub fn set_input(output: &Arc<Mutex<Output>>, new_input: String) {
    if let Ok(output) = output.lock() {
        output.set_input(new_input);
    }
}

pub fn print_prompt(output: &Arc<Mutex<Output>>) {
    if let Ok(output) = output.lock() {
        output.render_input();
    }
}
