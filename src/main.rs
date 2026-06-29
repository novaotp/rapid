use std::{
    io::{self, BufRead as _, Write as _},
    net,
};

fn main() -> io::Result<()> {
    let listener = net::TcpListener::bind("127.0.0.1:8080")?;

    for stream in listener.incoming() {
        handle_connection(stream?)?;
    }

    Ok(())
}

pub fn handle_connection(stream: net::TcpStream) -> io::Result<()> {
    let mut reader = io::BufReader::new(stream);

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
