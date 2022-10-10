use std::net::TcpStream;

use tungstenite::WebSocket;

pub enum WebSocketMessage {
    SaveFile { file_path: String, data: String },
    LoadFile { file_path: String },
    LoadProgram { name: String, contents: String },
    RunSC { program_name: String, to_convert: Vec<SCConversion> },
    Unknown,
}

pub enum WebSocketResponse {
    Success,
    Error { message: String },
    LoadFileResult { data: String },
    RunSCResult { to_convert: Vec<SCConversion> }
}

pub struct SCConversion {
    id: u32,
    data: String,
}

pub fn decode(raw_message: String) -> WebSocketMessage {
    print!("{}", raw_message);
    todo!();
}

impl WebSocketMessage {
    pub fn handle(&self) -> WebSocketResponse {
        match self {
            WebSocketMessage::SaveFile { file_path, data } => handle_save_file(file_path, data),
            WebSocketMessage::LoadFile { file_path } => todo!(),
            WebSocketMessage::LoadProgram { name, contents } => todo!(),
            WebSocketMessage::RunSC { program_name, to_convert } => todo!(),
            WebSocketMessage::Unknown => WebSocketResponse::Error { message: String::from("Unknown message") },
        } 
    }
}

impl WebSocketResponse {
    pub fn handle(&self) -> Option<String> {
        match self {
            WebSocketResponse::Success => todo!(),
            WebSocketResponse::Error { message } => todo!(),
            WebSocketResponse::LoadFileResult { data } => todo!(),
            WebSocketResponse::RunSCResult { to_convert } => todo!(),
        } 
    }
}

fn handle_save_file(file_path: &String, data: &String) -> WebSocketResponse {
    todo!()
}