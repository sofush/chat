use std::{
    io::{self, Write},
    net::{SocketAddr, TcpStream},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{message::Message, util};

struct ClientData {
    id: Option<String>,
}

pub struct Client {
    write: TcpStream,
    reader: JoinHandle<()>,
    data: Arc<Mutex<ClientData>>,
}

impl Client {
    pub fn new(addr: SocketAddr) -> io::Result<Self> {
        let write = TcpStream::connect_timeout(&addr, Duration::from_secs(5))?;
        let read = write.try_clone()?;
        let data = Arc::new(Mutex::new(ClientData { id: None }));
        let data_clone = data.clone();

        let cb = move |msg| {
            if let Message::AssignId(id) = &msg
                && let Ok(mut data) = data_clone.lock()
            {
                data.id = Some(id.to_owned());
                return;
            }

            println!("{msg:?}");
        };
        let reader =
            thread::spawn(move || util::read_from_stream(read, Box::new(cb)));

        let this = Self {
            write,
            reader,
            data,
        };
        Ok(this)
    }

    pub fn send(&mut self, message: Message) {
        if let Ok(str) = serde_json::to_string(&message) {
            let _ = writeln!(self.write, "{str}");
        }
    }
}
