use std::{
    io::{self, Write as _, stdout},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
};

use chrono::Local;
use crossterm::{
    cursor, execute,
    terminal::{self, ClearType},
};
use owo_colors::OwoColorize as _;

use crate::message::Message;

pub enum Label {
    Unlabeled,
    Error,
    Warn,
    Info,
    Debug,
    Input,
}

impl Label {
    fn get_ansi(&self) -> String {
        match self {
            Label::Warn => " WARN ".black().on_yellow().bold().to_string(),
            Label::Debug => {
                " DEBUG ".black().on_bright_black().bold().to_string()
            }
            Label::Input => {
                " INPUT ".black().on_bright_blue().bold().to_string()
            }
            Label::Unlabeled => "".to_string(),
            Label::Info => " INFO ".white().on_green().bold().to_string(),
            Label::Error => " ERROR ".black().on_red().bold().to_string(),
        }
    }
}

pub enum Printable {
    String { s: String, label: Label },
    Message { msg: Message, label: Label },
}

impl From<&'_ str> for Printable {
    fn from(value: &'_ str) -> Self {
        Self::String {
            s: value.to_string(),
            label: Label::Unlabeled,
        }
    }
}

impl From<String> for Printable {
    fn from(value: String) -> Self {
        Self::String {
            s: value,
            label: Label::Unlabeled,
        }
    }
}

impl From<Message> for Printable {
    fn from(value: Message) -> Self {
        Self::Message {
            msg: value,
            label: Label::Unlabeled,
        }
    }
}

fn print(s: &str, label: Label, input: &str, debug: bool) -> io::Result<()> {
    if matches!(label, Label::Debug) && !debug {
        return Ok(());
    }

    let now = format!(" {} ", Local::now().format("%Y-%m-%d %H:%M:%S"));
    let styled_now = now.black().on_yellow().bold().to_string();
    let mut width = ansi_width::ansi_width(&styled_now);

    clear_line()?;

    print!(" {styled_now} ");
    width += 2;

    if !s.is_empty() {
        let mut lines = s.lines();
        let label_str = label.get_ansi();
        width += ansi_width::ansi_width(&label_str);

        if let Some(first_line) = lines.next() {
            println!("{label_str} {first_line}");
        }

        let filler = " ".repeat(width);

        for line in lines {
            clear_line()?;
            println!("{filler} {line}");
        }
    }

    let input_label_ansi = Label::Input.get_ansi();
    print!("{} {}", input_label_ansi, input);
    stdout().flush()
}

fn print_message(
    msg: Message,
    label: Label,
    input: &str,
    debug: bool,
) -> io::Result<()> {
    let s = match &msg {
        Message::AnnounceJoin { id } => {
            format!("{} has joined the chat", id.clone().yellow())
        }
        Message::AssignId { id } => {
            format!("You have been assigned the ID: {}", id.clone().yellow())
        }
        _ => return Ok(()),
    };

    print(&s, label, input, debug)
}

fn clear_line() -> io::Result<()> {
    execute!(
        stdout(),
        cursor::MoveToColumn(0),
        terminal::Clear(ClearType::CurrentLine),
    )
}

#[allow(unused)]
pub struct Output {
    tx: mpsc::SyncSender<Printable>,
    th: JoinHandle<()>,
    input: Arc<Mutex<String>>,
    debug: Arc<AtomicBool>,
}

impl Output {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::sync_channel::<Printable>(1024);
        let input = Arc::new(Mutex::new(String::new()));
        let input_clone = input.clone();

        let debug = Arc::new(AtomicBool::new(false));
        let debug_clone = debug.clone();

        let th = thread::spawn(move || {
            loop {
                let Ok(printable) = rx.recv() else {
                    break;
                };

                let Ok(i) = input_clone.lock() else {
                    break;
                };

                let debug = debug_clone.load(Ordering::SeqCst);

                if let Printable::String { s, label } = printable {
                    let _ = print(&s, label, &i, debug);
                } else if let Printable::Message { msg, label } = printable {
                    let _ = print_message(msg, label, &i, debug);
                }

                let _ = stdout().flush();
            }
        });

        Self {
            tx,
            th,
            input,
            debug,
        }
    }

    pub fn print(&self, message: impl Into<Printable>) {
        let printable = message.into();
        let _ = self.tx.send(printable);
    }

    pub fn render_input(&self) {
        if let Ok(i) = self.input.lock() {
            let _ = print("", Label::Input, &i, false);
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
