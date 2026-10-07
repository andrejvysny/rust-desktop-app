use app_platform::CommandHost;
use app_ui_egui::control::{MAX_INPUT_BYTES, handle};
use serde_json::json;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};

const MAX_CLIENTS: usize = 16;

pub struct ControlServer {
    listener: TcpListener,
    clients: Vec<Client>,
}

impl ControlServer {
    pub fn bind(port: u16) -> io::Result<Self> {
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))?;
        listener.set_nonblocking(true)?;
        eprintln!("Control listening on {}", listener.local_addr()?);
        Ok(Self { listener, clients: Vec::new() })
    }

    pub fn poll(&mut self, host: &mut CommandHost) {
        for _ in 0..MAX_CLIENTS {
            match self.listener.accept() {
                Ok((stream, _)) if self.clients.len() < MAX_CLIENTS => {
                    if stream.set_nonblocking(true).is_ok() {
                        self.clients.push(Client::new(stream));
                    }
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
        self.clients.retain_mut(|client| client.poll(host));
    }
}

struct Client {
    stream: TcpStream,
    input: Vec<u8>,
    output: Vec<u8>,
    sent: usize,
    touched: Instant,
    closing: bool,
}

impl Client {
    fn new(stream: TcpStream) -> Self {
        Self { stream, input: Vec::new(), output: Vec::new(), sent: 0, touched: Instant::now(), closing: false }
    }

    fn poll(&mut self, host: &mut CommandHost) -> bool {
        if self.touched.elapsed() > Duration::from_secs(30) {
            return false;
        }
        if !self.flush() || self.closing && self.output.is_empty() {
            return false;
        }
        if !self.output.is_empty() {
            return true;
        }
        let mut bytes = [0_u8; 8192];
        let has_line = self.input.contains(&b'\n');
        let read_result = if has_line { Err(io::Error::from(io::ErrorKind::WouldBlock)) } else { self.stream.read(&mut bytes) };
        match read_result {
            Ok(0) => return false,
            Ok(count) => {
                self.input.extend_from_slice(bytes.get(..count).unwrap_or_default());
                self.touched = Instant::now();
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
            Err(_) => return false,
        }
        let frame_end = self.input.iter().position(|byte| *byte == b'\n');
        let frame_len = frame_end.unwrap_or(self.input.len());
        if frame_len > MAX_INPUT_BYTES {
            self.respond(json!({"error":{"code":"input_too_large","message":"request exceeds 1048576 bytes"}}));
            self.closing = true;
        } else if let Some(end) = frame_end {
            let line: Vec<u8> = self.input.drain(..=end).collect();
            let response = match serde_json::from_slice(&line) {
                Ok(request) => handle(host, &request),
                Err(error) => json!({"error":{"code":"invalid_json","message":error.to_string()}}),
            };
            self.respond(response);
        }
        true
    }

    fn respond(&mut self, response: serde_json::Value) {
        self.output = response.to_string().into_bytes();
        self.output.push(b'\n');
        self.sent = 0;
    }

    fn flush(&mut self) -> bool {
        if self.output.is_empty() {
            return true;
        }
        match self.stream.write(self.output.get(self.sent..).unwrap_or_default()) {
            Ok(0) => return false,
            Ok(count) => {
                self.sent = self.sent.saturating_add(count);
                self.touched = Instant::now();
                if self.sent == self.output.len() {
                    self.output.clear();
                }
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
            Err(_) => return false,
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead as _, BufReader};

    #[test]
    fn near_limit_frame_accepts_pipelined_next_request() {
        let mut server = ControlServer::bind(0).unwrap();
        let address = server.listener.local_addr().unwrap();
        let request = json!({"id":1,"method":"engine.execute","params":{"command":"demo.incrementCounter","params":{}}}).to_string();
        let mut bytes = vec![b' '; MAX_INPUT_BYTES - request.len()];
        bytes.extend_from_slice(request.as_bytes());
        bytes.extend_from_slice(b"\n{\"id\":2,\"method\":\"state.get\"}\n");
        let client = std::thread::spawn(move || {
            let mut stream = TcpStream::connect(address).unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
            stream.write_all(&bytes).unwrap();
            stream.shutdown(std::net::Shutdown::Write).unwrap();
            let mut reader = BufReader::new(stream);
            let mut replies = Vec::new();
            for _ in 0..2 {
                let mut line = String::new();
                assert!(reader.read_line(&mut line).unwrap() > 0);
                replies.push(serde_json::from_str::<serde_json::Value>(&line).unwrap());
            }
            replies
        });
        let mut host = CommandHost::default();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !client.is_finished() && Instant::now() < deadline {
            server.poll(&mut host);
            std::thread::sleep(Duration::from_micros(100));
        }
        let replies = client.join().unwrap();
        assert_eq!(replies.first().unwrap().get("id"), Some(&json!(1)));
        assert!(replies.iter().all(|reply| reply.get("result").is_some()));
        assert_eq!(host.session().model().counter, 1);
    }
}
