use std::{
    io::{self, BufReader, Write as _},
    net::TcpListener,
};

use prime_http::{
    request::{Request, RequestParseError},
    response::{Response, StringResponseBodyPayload},
    status_code::StatusCode,
};

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut reader = BufReader::new(&stream);

                let request = Request::parse(&mut reader);
                match request {
                    Ok(request) => {
                        println!("{:#?}", request);

                        let response = "HTTP/1.1 200 OK\r\n\r\n";
                        stream.write_all(response.as_bytes())?;
                        stream.flush()?;
                    }
                    Err(e) => {
                        let response = Response::default();

                        match e {
                            RequestParseError::LengthRequired => {
                                response.status = StatusCode::LengthRequired;
                                response.set_header("Content-Type", "text/plain");
                                response.set_body(Some(Box::new(StringResponseBodyPayload::new(
                                    "hello world",
                                ))));

                                let message = "Request must have a Content-Length header.";

                                format!(
                                    "HTTP/1.1 411 Length Required\r\n\
                                     Content-Type: text/plain\r\n\
                                     Content-Length: {}\r\n\
                                     \r\n\
                                     {}\r\n\
                                     \r\n",
                                    message.len(),
                                    message
                                )
                            }
                            RequestParseError::UnsupportedContentType => {
                                let message =
                                    "Only `Content-Type: text/plain` is supported for now.";

                                format!(
                                    "HTTP/1.1 501 Not Implemented\r\n\
                                     Content-Type: text/plain\r\n\
                                     Content-Length: {}\r\n\
                                     \r\n\
                                     {}\r\n\
                                     \r\n",
                                    message.len(),
                                    message
                                )
                            }
                            RequestParseError::UnsupportedTransferEncoding(_) => {
                                let message = "Header `Transfer-Encoding` is not yet supported.";

                                format!(
                                    "HTTP/1.1 501 Not Implemented\r\n\
                                     Content-Type: text/plain\r\n\
                                     Content-Length: {}\r\n\
                                     \r\n\
                                     {}\r\n\
                                     \r\n",
                                    message.len(),
                                    message
                                )
                            }
                            _ => "HTTP/1.1 400 Bad Request\r\n\r\n".to_owned(),
                        };

                        stream.write_all(response.as_bytes())?;
                        stream.flush()?;
                    }
                }
            }
            Err(e) => eprintln!("An error occurred on the stream : {:#?}", e),
        }
    }

    Ok(())
}
