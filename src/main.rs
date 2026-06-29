use std::{
    io::{self, BufReader, Write as _},
    net::{Shutdown, TcpListener},
};

use prime_server::request::Request;

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut reader = BufReader::new(&stream);

                let request = Request::parse(&mut reader);
                match request {
                    Ok(request) => println!("{:#?}", request),
                    Err(e) => {
                        eprintln!(
                            "An error occurred while parsing the HTTP request : {:#?}",
                            e
                        )
                    }
                }

                // After parsing request, no need to read anymore
                stream.shutdown(Shutdown::Read)?;

                let response = "HTTP/1.1 200 OK\r\n\r\n";
                stream.write_all(response.as_bytes())?;

                stream.flush()?;
                stream.shutdown(Shutdown::Write)?;
            }
            Err(e) => eprintln!("An error occurred on the stream : {:#?}", e),
        }
    }

    Ok(())
}
