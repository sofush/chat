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

use crate::server::Server;

mod client;
mod message;
mod participant;
mod server;

fn main() -> io::Result<()> {
    let mut stdout = stdout();

    terminal::enable_raw_mode()?;
    let mut input = String::new();

    let addr = SocketAddr::from_str("127.0.0.1:3000").unwrap();
    let mut server = Server::new(addr)?;
    server.host()?;

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
                        println!("{input}");
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
