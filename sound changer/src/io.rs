use std::{fs::File, net::TcpStream};
use std::io::prelude::*;
use std::path::Path;
use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::{accept, Message, WebSocket};
use no_panic::no_panic;
use super::websocket_handler::*;

#[derive(Debug)]
pub enum IOError {
    FileNotFound(String),
    FileExists(String),
    InvalidFilePath(String),
    ReadError(String),
    WriteError(String),
    Other(String),
}

//https://doc.rust-lang.org/rust-by-example/std_misc/file/open.html
#[no_panic]
pub fn load_from_file(path_str: &String) -> Result<String, IOError> {
    use std::time::Instant;
    let now = Instant::now();

    let path = Path::new(path_str);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => return Err(IOError::FileNotFound(format!("couldn't open {}: {}", display, why))),
        Ok(file) => file,
    };

    let mut s = String::new();
    let temp = match file.read_to_string(&mut s) {
        Err(why) => return Err(IOError::ReadError(format!("couldn't read {}: {}", display, why))),
        Ok(_) => s,
    };

    let elapsed = now.elapsed();
    print!("Loaded file in {:.2?}\n", elapsed);

    return Ok(temp);
}

pub fn save_to_file(path_str: &String, data: &String, overwrite: bool, restrict_path: bool) -> Option<IOError> {
    use std::time::Instant;
    let now = Instant::now();

    if restrict_path {
        if path_str.contains(":") || path_str.contains("..") {
            return Some(IOError::InvalidFilePath(format!("security settings do not allow the path: {}", path_str)));
        }
    }

    let path = Path::new(path_str);
    let display = path.display();

    if path.exists() {
        if !overwrite {
            return Some(IOError::FileExists(format!("file already exists: {}", display)));
        }
    } else {
        if overwrite {
            return Some(IOError::FileNotFound(format!("file doesn't exist but overwrite still specified: {}", display)));
        }
    }

    let mut file = match File::create(&path) {
        Err(why) => return Some(IOError::FileNotFound(format!("couldn't create {}: {}", display, why))),
        Ok(file) => file,
    };

    match file.write_all(data.as_bytes()) {
        Err(why) => return Some(IOError::WriteError(format!("couldn't write {}: {}", display, why))),
        Ok(_) => (),
    };

    let elapsed = now.elapsed();
    print!("Saved file file in {:.2?}\n", elapsed);

    return None;
}

pub fn web_socket_listener() {
    let server = TcpListener::bind("127.0.0.1:9001").unwrap();
    for stream in server.incoming() {
        spawn (move || {
            let websocket_result = accept(stream.unwrap());
            let mut websocket: WebSocket<TcpStream> = match websocket_result {
                Ok(v) => v,
                Err(v) => {
                    println!("{}", v.to_string());
                    return;
                }
            };
            println!("Successfully opened websocket");
            loop {
                let msg = websocket.read_message().unwrap();

                use std::time::Instant;
                let now = Instant::now();

                let message = decode(msg.to_string());
                let response = message.handle();
                let response_message = response.handle();
                match response_message {
                    Some(msg) => push_messages(&mut websocket, msg),
                    None => panic!("Couldn't serialize response"),
                };
                
                let elapsed = now.elapsed();
                print!("Handled message in : {:.2?}\n", elapsed);
            }
        });
    }
}

pub fn push_messages(websocket: &mut WebSocket<TcpStream>, message: String) -> Option<String> {
    let msg: Message = Message::Text(message);
    match websocket.write_message(msg) {
        Ok(_) => None,
        Err(msg) => Some(format!("Could not send message: {}", msg)),
    }
}