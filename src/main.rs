use std::io::{self, Write, stdout};
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;

use crossterm::event::KeyModifiers;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, ClearType},
};

use crate::client::Client;
use crate::server::Server;

mod client;
mod message;
mod participant;
mod server;
mod util;

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;

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
            )? {
                break;
            }
        }

        render_input(&input)?;
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
) -> io::Result<bool> {
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            print!("^C");
            return Ok(true);
        }
        KeyCode::Char(c) => input.push(c),
        KeyCode::Backspace => {
            input.pop();
        }
        KeyCode::Enter => {
            return handle_enter(input, server, client, addr);
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
) -> io::Result<bool> {
    if input.trim() == "exit" {
        return Ok(true);
    }

    clear_line()?;
    handle_command(input, server, client, addr)?;
    input.clear();
    Ok(false)
}

fn handle_command(
    input: &str,
    server: &mut Option<Server>,
    client: &mut Option<Client>,
    addr: SocketAddr,
) -> io::Result<()> {
    if try_host(input, server, addr)? {
        return Ok(());
    }

    if try_connect(input, client, addr)? {
        return Ok(());
    }

    if let Some(c) = client {
        c.send(message::Message::User(input.to_string()));
        clear_line()?;
        println!("{input}");
        return Ok(());
    }

    clear_line()?;
    println!("You must connect to a server before sending messages.");
    Ok(())
}

fn try_host(
    input: &str,
    server: &mut Option<Server>,
    addr: SocketAddr,
) -> io::Result<bool> {
    if !input.starts_with("/host") || server.is_some() {
        return Ok(false);
    }

    if let Ok(mut srv) = Server::new(addr) {
        srv.host()?;
        *server = Some(srv);

        clear_line()?;
        println!("Listening on {addr}!");
    }

    Ok(true)
}

fn try_connect(
    input: &str,
    client: &mut Option<Client>,
    addr: SocketAddr,
) -> io::Result<bool> {
    if !input.starts_with("/connect") || client.is_some() {
        return Ok(false);
    }

    println!("Connecting to {addr}...");

    if let Ok(c) = Client::new(addr) {
        *client = Some(c);
    }

    clear_line()?;

    let status = if client.is_some() {
        "Connected!"
    } else {
        "Could not connect."
    };

    println!("{status}");

    Ok(true)
}

fn render_input(input: &str) -> io::Result<()> {
    clear_line()?;
    print!("> {}", input);
    stdout().flush()
}

fn clear_line() -> io::Result<()> {
    execute!(
        stdout(),
        cursor::MoveToColumn(0),
        terminal::Clear(ClearType::CurrentLine),
    )
}
