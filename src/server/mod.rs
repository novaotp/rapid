pub mod method;

use std::{
    collections::HashMap,
    io::{self, BufRead as _},
    net,
    str::FromStr as _,
};

use crate::{request::Request, response::Response, server::method::Method};

/// Various options for initializing a server.
pub struct CreateServerOptions<'a> {
    /// The IP address on which to listen.
    ip_address: &'a str,
    /// The port to listen to.
    port: u16,
}

impl Default for CreateServerOptions<'_> {
    fn default() -> Self {
        CreateServerOptions {
            ip_address: "127.0.0.1",
            port: 8080,
        }
    }
}

pub struct Server {
    listener: net::TcpListener,
    callbacks: HashMap<Method, fn(request: &mut Request, response: &mut Response)>,
}

impl Server {
    /// Creates a new HTTP server with the given options.
    pub fn new<'a>(options: CreateServerOptions<'a>) -> Result<Self, io::Error> {
        let addr = format!("{}:{}", options.ip_address, options.port);
        let listener = net::TcpListener::bind(addr)?;

        Ok(Server {
            listener,
            callbacks: HashMap::new(),
        })
    }

    /// Starts listening to incoming TCP streams.
    pub fn listen(&self) -> io::Result<()> {
        for stream in self.listener.incoming() {
            self.handle_connection(stream?)?;
        }

        Ok(())
    }

    /// Registers a new route with GET method.
    pub fn get(
        &mut self,
        _path: &str,
        callback: fn(request: &mut Request, response: &mut Response),
    ) -> () {
        self.callbacks.insert(Method::GET, callback);
    }

    fn handle_connection(&self, stream: net::TcpStream) -> io::Result<()> {
        let mut reader = io::BufReader::new(stream);

        let mut line = String::new();
        reader.read_line(&mut line);

        let mut parts = line.split(' ');

        let method = parts.next().unwrap();
        let path = parts.next().unwrap();
        let version = parts.next().unwrap();

        let headers: HashMap<String, String> = HashMap::new();
        for l in (&mut reader).lines() {
            let split = l?.split_once(": ");
            if split.is_none() {
                eprintln!("An error occurred while reading header.");
                continue;
            }

            let (key, val) = split.unwrap();
            headers.insert(key.to_string(), val.to_string());

            // Empty line means we reached \r\n\r\n
            if l? == "" {
                break;
            }
        }

        let request = Request {
            body: String::new(),
        };
        let response = Response::new(&stream);

        let callback = self.callbacks.get(&Method::from_str(method).unwrap());

        Ok(())
    }
}
