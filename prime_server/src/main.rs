use std::{
    io::{self, BufReader, Write as _},
    net::TcpListener,
};

use prime_http::{
    request::{Request, RequestParseError},
    response::Response,
    status_code::StatusCode,
};
use prime_middlewares::string::StringResponseBodyPayload;

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut reader = BufReader::new(&stream);

                let request = Request::parse(&mut reader);
                let response = create_response(&request);

                stream.write_all(response.to_string().as_bytes())?;
                stream.flush()?;
            }
            Err(e) => eprintln!("An error occurred on the stream : {:#?}", e),
        }
    }

    Ok(())
}

fn create_response(request: &Result<Request, RequestParseError>) -> Response {
    let mut response = Response::default();

    match request {
        Ok(request) => {
            println!("{:#?}", request);
        }
        Err(e) => {
            match e {
                RequestParseError::LengthRequired => {
                    response.status = StatusCode::LengthRequired;
                    response.set_body(Box::new(StringResponseBodyPayload::new(
                        "Request must have a Content-Length header.",
                    )));
                }
                RequestParseError::UnsupportedContentType => {
                    response.status = StatusCode::NotImplemented;
                    response.set_body(Box::new(StringResponseBodyPayload::new(
                        "Only `Content-Type: text/plain` is supported for now.",
                    )));
                }
                RequestParseError::UnsupportedTransferEncoding(_) => {
                    response.status = StatusCode::NotImplemented;
                    response.set_body(Box::new(StringResponseBodyPayload::new(
                        "Header `Transfer-Encoding` is not yet supported.",
                    )));
                }
                _ => {
                    response.status = StatusCode::BadRequest;
                }
            };
        }
    }

    response
}
