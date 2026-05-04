use std::io::{self, Write, stdout};
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;

use crossterm::event::KeyModifiers;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
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
    let mut stdout = stdout();

    terminal::enable_raw_mode()?;
    let mut input = String::new();

    let addr = SocketAddr::from_str("127.0.0.1:3000").unwrap();
    let mut server: Option<Server> = None;
    let mut client: Option<Client> = None;

    loop {
        if event::poll(Duration::from_millis(16))?
            && let Event::Key(key) = event::read()?
        {
            match key.code {
                KeyCode::Char('c')
                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    print!("^C");
                    break;
                }
                KeyCode::Char(c) => input.push(c),
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Enter => {
                    if input.trim() == "exit" {
                        break;
                    } else {
                        execute!(
                            stdout,
                            cursor::MoveToColumn(0),
                            terminal::Clear(ClearType::CurrentLine),
                        )?;

                        if input.starts_with("/host") && server.is_none() {
                            if let Ok(mut srv) = Server::new(addr) {
                                srv.host()?;
                                server = Some(srv);
                                execute!(
                                    stdout,
                                    cursor::MoveToColumn(0),
                                    terminal::Clear(ClearType::CurrentLine),
                                )?;
                                println!("Listening on {addr}!");
                            }
                        } else if input.starts_with("/connect")
                            && client.is_none()
                        {
                            println!("Connecting to {addr}...");

                            if let Ok(c) = Client::new(addr) {
                                client = Some(c);
                            }

                            execute!(
                                stdout,
                                cursor::MoveToColumn(0),
                                terminal::Clear(ClearType::CurrentLine),
                            )?;

                            let status = if client.is_some() {
                                "Connected!"
                            } else {
                                "Could not connect."
                            };

                            println!("{status}");
                        } else {
                            if let Some(c) = &mut client {
                                c.send(message::Message::User());
                                execute!(
                                    stdout,
                                    cursor::MoveToColumn(0),
                                    terminal::Clear(ClearType::CurrentLine),
                                )?;
                                println!("{input}");
                            } else {
                                execute!(
                                    stdout,
                                    cursor::MoveToColumn(0),
                                    terminal::Clear(ClearType::CurrentLine),
                                )?;
                                println!(
                                    "You must connect to a server before sending messages."
                                );
                            }
                        }
                    }
                    input.clear();
                }
                _ => {}
            }
        }

        execute!(
            stdout,
            cursor::MoveToColumn(0),
            terminal::Clear(ClearType::CurrentLine),
        )?;

        print!("> {}", input);
        stdout.flush()?;
    }

    terminal::disable_raw_mode()?;
    Ok(())
}
