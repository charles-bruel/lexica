use crate::sc::data::create_thread_context;
use std::io::prelude::*;
use std::net::TcpListener;
use std::path::Path;
use std::thread::spawn;
use std::{fs::File, net::TcpStream};
use tungstenite::{accept, Message, WebSocket};

use super::websocket_handler::*;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum IOError {
    FileNotFound(String),
    FileExists(String),
    InvalidFilePath(String),
    ReadError(String),
    WriteError(String),
    Other(String),
}

impl IOError {
    pub fn get_response(&self) -> WebSocketResponse {
        WebSocketResponse::Error {
            message: self.get_message().clone(),
        }
    }

    pub fn get_message(&self) -> &String {
        match &self {
            IOError::FileNotFound(err) => err,
            IOError::ReadError(err) => err,
            IOError::InvalidFilePath(err) => err,
            IOError::WriteError(err) => err,
            IOError::Other(err) => err,
            IOError::FileExists(err) => err,
        }
    }
}

//https://doc.rust-lang.org/rust-by-example/std_misc/file/open.html
pub fn load_from_file(path_str: &String, restrict_path: bool) -> Result<String, IOError> {
    use std::time::Instant;
    let now = Instant::now();

    if restrict_path && (path_str.contains(':') || path_str.contains("..")) {
        return Err(IOError::InvalidFilePath(format!(
            "security settings do not allow the path: {}",
            path_str
        )));
    }

    let path = Path::new(path_str);
    let display = path.display();

    let mut file = match File::open(path) {
        Err(why) => {
            return Err(IOError::FileNotFound(format!(
                "couldn't open {}: {}",
                display, why
            )))
        }
        Ok(file) => file,
    };

    let mut s = String::new();
    let temp = match file.read_to_string(&mut s) {
        Err(why) => {
            return Err(IOError::ReadError(format!(
                "couldn't read {}: {}",
                display, why
            )))
        }
        Ok(_) => s,
    };

    let elapsed = now.elapsed();
    println!("Loaded file in {:.2?}", elapsed);

    Ok(temp)
}

pub fn save_to_file(
    path_str: &String,
    data: &String,
    overwrite: bool,
    restrict_path: bool,
) -> Option<IOError> {
    use std::time::Instant;
    let now = Instant::now();

    if restrict_path && (path_str.contains(':') || path_str.contains("..")) {
        return Some(IOError::InvalidFilePath(format!(
            "security settings do not allow the path: {}",
            path_str
        )));
    }

    let path = Path::new(path_str);
    let display = path.display();

    if path.exists() && !overwrite {
        return Some(IOError::FileExists(format!(
            "file already exists: {}",
            display
        )));
    }

    let mut file = match File::create(path) {
        Err(why) => {
            return Some(IOError::FileNotFound(format!(
                "couldn't create {}: {}",
                display, why
            )))
        }
        Ok(file) => file,
    };

    if let Err(why) = file.write_all(data.as_bytes()) {
        return Some(IOError::WriteError(format!(
            "couldn't write {}: {}",
            display, why
        )));
    }

    let elapsed = now.elapsed();
    println!("Saved file file in {:.2?}", elapsed);

    None
}

pub fn web_socket_listener() {
    let server = TcpListener::bind("127.0.0.1:9001").unwrap();
    for stream in server.incoming() {
        spawn(move || {
            let websocket_result = accept(stream.unwrap());
            let websocket: WebSocket<TcpStream> = match websocket_result {
                Ok(v) => v,
                Err(v) => {
                    println!("{}", v.to_string());
                    return;
                }
            };
            println!("Successfully opened websocket");

            web_socket_loop(websocket);
            println!("Thread closed")
        });
    }
}

fn web_socket_loop(mut websocket: WebSocket<TcpStream>) {
    let mut context = create_thread_context();

    loop {
        let temp = websocket.read();
        let msg = match temp {
            Ok(v) => v,
            Err(_) => break,
        };

        use std::time::Instant;
        let now = Instant::now();

        let message = decode(msg.to_string());
        let responses = message.handle(&mut context);
        for response in responses {
            let response_message = response.handle();
            match response_message {
                Some(msg) => push_messages(&mut websocket, msg),
                None => panic!("Couldn't serialize response"),
            };
        }
        while !context.queued_extra_messages.is_empty() {
            let extra_message = context.queued_extra_messages.pop_front().unwrap().handle();
            match extra_message {
                Some(msg) => push_messages(&mut websocket, msg),
                None => panic!("Couldn't serialize response"),
            };
        }

        let elapsed = now.elapsed();
        match message {
            WebSocketMessage::SaveFile {
                file_path: _,
                data: _,
                overwrite: _,
            } => println!("Handled save file message in: {:.2?}", elapsed),
            WebSocketMessage::LoadFile { file_path: _ } => {
                println!("Handled load file message in: {:.2?}", elapsed)
            }
            WebSocketMessage::LoadProgram {
                name: _,
                contents: _,
            } => println!("Handled load program message in: {:.2?}", elapsed),
            WebSocketMessage::TryCompile { program: _ } => {
                println!("Handled try compile message in: {:.2?}", elapsed)
            }
            WebSocketMessage::RunSC {
                program_name: _,
                to_convert: _,
            } => println!("Handled run sound changer message in: {:.2?}", elapsed),
            WebSocketMessage::Unknown { error: _ } => {
                println!("Handled unknown message in: {:.2?}", elapsed)
            }
            WebSocketMessage::LoadTable { contents: _ } => {
                println!("Handled load table message in: {:.2?}", elapsed)
            }
            WebSocketMessage::RebuildTables { start_index: _ } => {
                println!("Handled rebuild tables message in: {:.2?}", elapsed)
            }
        }
    }
}

pub fn push_messages(websocket: &mut WebSocket<TcpStream>, message: String) -> Option<String> {
    let msg: Message = Message::Text(message);
    match websocket.send(msg) {
        Ok(_) => None,
        Err(msg) => Some(format!("Could not send message: {}", msg)),
    }
}
