use std::{
    io::{self, Write as _, stdout},
    sync::{Arc, Mutex, mpsc},
    thread::{self, JoinHandle},
};

use chrono::Local;
use crossterm::{
    cursor, execute,
    style::Stylize,
    terminal::{self, ClearType},
};

use crate::message::Message;

pub enum Printable {
    String(String),
    Message(Message),
}

impl From<&'_ str> for Printable {
    fn from(value: &'_ str) -> Self {
        Self::String(value.to_string())
    }
}

impl<'a> From<String> for Printable {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<Message> for Printable {
    fn from(value: Message) -> Self {
        Self::Message(value)
    }
}

fn print(s: &str, input: &str) -> io::Result<()> {
    let now = format!(
        " {} ",
        Local::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
    )
    .on_dark_blue()
    .bold();

    clear_line()?;

    print!(" {now} ");

    if !s.is_empty() {
        println!("{s}");
    }

    print!("{} {}", " INPUT ".bold().black().on_white(), input);
    stdout().flush()
}

fn print_message(msg: Message, input: &str) -> io::Result<()> {
    let s = match &msg {
        Message::User(u) => u.as_str(),
        Message::AssignId(id) => &format!("You have been assigned ID: {id}"),
    };

    print(s, input)
}

fn clear_line() -> io::Result<()> {
    execute!(
        stdout(),
        cursor::MoveToColumn(0),
        terminal::Clear(ClearType::CurrentLine),
    )
}

pub struct Output {
    tx: mpsc::SyncSender<Printable>,
    th: JoinHandle<()>,
    input: Arc<Mutex<String>>,
}

impl Output {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::sync_channel::<Printable>(1024);
        let input = Arc::new(Mutex::new(String::new()));
        let input_clone = input.clone();

        let th = thread::spawn(move || {
            loop {
                let Ok(printable) = rx.recv() else {
                    break;
                };

                let Ok(i) = input_clone.lock() else {
                    break;
                };

                if let Printable::String(str) = printable {
                    let _ = print(&str, &i);
                } else if let Printable::Message(msg) = printable {
                    let _ = print_message(msg, &i);
                }

                let _ = stdout().flush();
            }
        });

        Self { tx, th, input }
    }

    pub fn print<'a>(&self, message: impl Into<Printable>) {
        let printable = message.into();
        let _ = self.tx.send(printable);
    }

    pub fn render_input(&self) {
        if let Ok(i) = self.input.lock() {
            let _ = print("", &i);
        }
    }

    pub fn clear_input(&self) {
        self.set_input("".to_string())
    }

    pub fn set_input(&self, new_input: String) {
        if let Ok(mut i) = self.input.lock() {
            *i = new_input;
        }

        self.render_input()
    }
}
