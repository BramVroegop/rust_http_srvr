use std::{error::Error, fmt::Display, fs, io};

use chrono::Utc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

#[derive(Debug)]
struct HttpError {
    message: String,
}

impl Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for HttpError {}

pub struct SRVR;

struct GetReq {
    route: String,
}

enum HttpMethod {
    Get(GetReq),
}

impl SRVR {
    pub fn new() -> Self {
        Self
    }

    #[tokio::main]
    pub async fn listen(&self, ip: &str, port: &str) -> Result<(), io::Error> {
        println!("Listening on port: {port}");
        let listener = TcpListener::bind(format!("{ip}:{port}")).await?;

        loop {
            let (socket, _) = listener.accept().await?;
            tokio::spawn(async move {
                let res = SRVR::handle_connection(socket).await;

                match res {
                    Ok(e) => {}
                    Err(e) => return,
                }
            });
        }
    }

    async fn handle_connection(mut socket: tokio::net::TcpStream) -> Result<(), io::Error> {
        let mut buf = [0; 1024];
        let mut message = String::new();

        loop {
            match socket.read(&mut buf).await {
                Ok(n) if n == 0 => return Ok(()),
                Ok(n) => {
                    let s = String::from_utf8_lossy(&buf[..n]);
                    message.push_str(&s);
                }
                Err(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Error reading connection buffer.",
                    ))
                }
            };
            let request = SRVR::decode_message(&message);

            if let Ok(r) = request {
                let res = &SRVR::encode_message(r);
                if let Err(e) = socket.write_all(&res.as_bytes()).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                }
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Error decoding request.",
                ));
            }
        }
    }

    fn decode_message(bytes_received: &String) -> Result<HttpMethod, io::Error> {
        let mut headers = bytes_received.split("\r\n");

        let first_line = headers.next().unwrap_or("");

        if first_line.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to parse method header.",
            ));
        }

        let mut route_headers = first_line.split(" ");
        route_headers.next();

        let request = GetReq {
            route: String::from(route_headers.next().unwrap_or("")),
        };

        Ok(HttpMethod::Get(request))
    }

    fn encode_message(request: HttpMethod) -> String {
        let mut response = String::from("HTTP/1.1 200 OK\r\n");

        let now = Utc::now();
        let http_date = now.format("%a, %d %b %Y %H:%M:%S GMT");

        response.push_str(&format!("Date: {}\r\n", http_date)); // Fix typo
        response.push_str("Content-Type: text/html; charset=UTF-8\r\n");
        match request {
            HttpMethod::Get(r) => {
                if r.route == "/" {
                    let file_buf =
                        fs::read_to_string("./pages/index.html").unwrap_or(String::from(""));
                    response.push_str(&format!(
                        "Content-Length: {}\r\n\r\n{}", // Fix typo and add newline after headers
                        file_buf.len(),
                        &file_buf
                    ));
                }
            }
            _ => {}
        }
        response
    }
}
