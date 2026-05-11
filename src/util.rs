use std::{
    io::BufReader,
    net::TcpStream,
    sync::{Arc, Mutex},
};

use owo_colors::OwoColorize as _;
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

pub fn warn(output: &Arc<Mutex<Output>>, message: impl Into<Printable>) {
    if let Ok(output) = output.lock() {
        let warn = " WARN ".black().on_yellow().to_string();
        let printable: Printable = message.into();

        let s = match printable {
            Printable::String(s) => s,
            Printable::Message(message) => format!("{message:?}"),
        };

        output.print(format!("{warn} {}", s.yellow()));
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

pub fn ctrl_w_delete(input: &str) -> String {
    let mut chars: Vec<char> = input.chars().collect();

    if chars.is_empty() {
        return String::new();
    }

    #[derive(PartialEq)]
    enum Kind {
        Whitespace,
        Keyword,
        Punctuation,
    }

    fn classify(c: char) -> Kind {
        if c.is_whitespace() {
            Kind::Whitespace
        } else if c.is_alphanumeric() || c == '_' {
            Kind::Keyword
        } else {
            Kind::Punctuation
        }
    }

    let last_kind = classify(*chars.last().unwrap());

    while let Some(&c) = chars.last() {
        if classify(c) == last_kind {
            chars.pop();
        } else {
            break;
        }
    }

    chars.into_iter().collect()
}
