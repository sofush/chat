use std::io::{self, Write, stdout};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::event::KeyModifiers;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, ClearType},
};

use crate::client::Client;
use crate::output::Output;
use crate::server::Server;

mod client;
mod message;
mod output;
mod participant;
mod server;
mod util;

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;

    let output = Arc::new(Mutex::new(Output::new()));

    let addr = SocketAddr::from_str("127.0.0.1:3000").unwrap();
    let mut input = String::new();
    let mut server: Option<Server> = None;
    let mut client: Option<Client> = None;

    loop {
        if let Some(key) = poll_key_event()? {
            if handle_key_event(
                key,
                &mut input,
                &mut server,
                &mut client,
                addr,
                output.clone(),
            )? {
                break;
            }
        }

        util::print_prompt(&output);
    }

    terminal::disable_raw_mode()?;
    Ok(())
}

fn poll_key_event() -> io::Result<Option<KeyEvent>> {
    if !event::poll(Duration::from_millis(16))? {
        return Ok(None);
    }

    match event::read()? {
        Event::Key(key) => Ok(Some(key)),
        _ => Ok(None),
    }
}

fn handle_key_event(
    key: KeyEvent,
    input: &mut String,
    server: &mut Option<Server>,
    client: &mut Option<Client>,
    addr: SocketAddr,
    output: Arc<Mutex<Output>>,
) -> io::Result<bool> {
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            print!("^C");
            return Ok(true);
        }
        KeyCode::Char(c) => {
            input.push(c);
            util::set_input(&output, input.to_string());
        }
        KeyCode::Backspace => {
            input.pop();
            util::set_input(&output, input.to_string());
        }
        KeyCode::Enter => {
            return handle_enter(input, server, client, addr, output.clone());
        }
        _ => {}
    }

    Ok(false)
}

fn handle_enter(
    input: &mut String,
    server: &mut Option<Server>,
    client: &mut Option<Client>,
    addr: SocketAddr,
    output: Arc<Mutex<Output>>,
) -> io::Result<bool> {
    if input.trim() == "exit" {
        return Ok(true);
    }

    handle_command(input, server, client, addr, output.clone())?;
    input.clear();
    util::clear_input(&output);
    Ok(false)
}

fn handle_command(
    input: &str,
    server: &mut Option<Server>,
    client: &mut Option<Client>,
    addr: SocketAddr,
    output: Arc<Mutex<Output>>,
) -> io::Result<()> {
    if try_host(input, server, addr, output.clone())? {
        return Ok(());
    }

    if try_connect(input, client, addr, output.clone())? {
        return Ok(());
    }

    if input.starts_with("/") {
        util::print(&output, format!("Unrecognized command: {input}"));
        return Ok(());
    }

    if let Some(c) = client {
        c.send(message::Message::User(input.to_string()));
        util::print(&output, input);
        return Ok(());
    }

    util::print(
        &output,
        "You must connect to a server before sending messages.",
    );

    Ok(())
}

fn try_host(
    input: &str,
    server: &mut Option<Server>,
    addr: SocketAddr,
    output: Arc<Mutex<Output>>,
) -> io::Result<bool> {
    if !input.starts_with("/host") || server.is_some() {
        return Ok(false);
    }

    if let Ok(mut srv) = Server::new(addr, output.clone()) {
        srv.host()?;
        *server = Some(srv);
        util::print(&output, format!("Listening on {addr}!"));
    }

    Ok(true)
}

fn try_connect(
    input: &str,
    client: &mut Option<Client>,
    addr: SocketAddr,
    output: Arc<Mutex<Output>>,
) -> io::Result<bool> {
    if !input.starts_with("/connect") || client.is_some() {
        return Ok(false);
    }

    util::print(&output, format!("Connecting to {addr}..."));

    if let Ok(c) = Client::new(addr, output.clone()) {
        *client = Some(c);
    }

    let status = if client.is_some() {
        "Connected!"
    } else {
        "Could not connect."
    };

    util::print(&output, status);
    Ok(true)
}
