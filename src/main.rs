pub mod request;
pub mod response;
pub mod server;
pub mod status;

use crate::{
    server::{CreateServerOptions, Server},
    status::Status,
};
use std::{
    io::{self, BufRead, BufReader, Error, Write},
    net,
};

fn main() -> Result<(), io::Error> {
    let mut server = Server::new(CreateServerOptions::default())?;

    server.get("/", |_request, response| {
        response.set_header("Content-Type".to_string(), "text/plain".to_string());

        response.set_status(Status::OK);
        response.set_body("hello world".to_string());

        if let Err(e) = response.send() {
            eprintln!("{}", e);
        }
    });

    if let Err(e) = server.listen() {
        eprintln!("{}", e);
    }

    Ok(())
}

pub fn handle_connection(stream: net::TcpStream) -> Result<(), Error> {
    let mut reader = BufReader::new(stream);

    for line in (&mut reader).lines() {
        let line = line?;
        println!("{line}");

        // Empty line means we reached \r\n\r\n
        if line == "" {
            break;
        }
    }

    let mut s = reader.into_inner();
    s.shutdown(net::Shutdown::Read)?;

    let response = "HTTP/1.1 200 OK\r\n\r\n";
    s.write_all(response.as_bytes())?;

    s.flush()?;
    s.shutdown(net::Shutdown::Write)?;

    Ok(())
}
