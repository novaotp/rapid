/*
 * Copyright 2026 Sajidur Rahman
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::{
    io::{self, BufReader, Write as _},
    net::TcpListener,
};

use rapid_http::request::{Request, RequestError};

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut reader = BufReader::new(&stream);

                let request = Request::from_reader(&mut reader);
                println!("{:#?}", request);

                let response = match request {
                    Ok(_) => String::from("HTTP/1.1 204 No Content\r\n\r\n"),
                    Err(e) => match e {
                        RequestError::UnsupportedHttpVersion => {
                            String::from("HTTP/1.1 505 HTTP Version Not Supported\r\n\r\n")
                        }
                        RequestError::InvalidMethod => {
                            String::from("HTTP/1.1 405 Method Not Allowed\r\n\r\n")
                        }
                        RequestError::Read(_) => String::from("HTTP/1.1 400 Bad Request\r\n\r\n"),
                    },
                };
                println!("{:#?}", response);

                stream.write_all(response.as_bytes())?;
                stream.flush()?;
            }
            Err(e) => eprintln!("An error occurred on the stream : {:#?}", e),
        }
    }

    Ok(())
}
