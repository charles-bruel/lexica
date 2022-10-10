use serde::{Serialize, Deserialize};
use super::io::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum WebSocketMessage {
    SaveFile { file_path: String, data: String, overwrite: bool },
    LoadFile { file_path: String },
    LoadProgram { name: String, contents: String },
    RunSC { program_name: String, to_convert: Vec<SCConversion> },
    Unknown { error: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum WebSocketResponse {
    Success,
    Error { message: String },
    RequestOverwrite,
    LoadFileResult { data: String },
    RunSCResult { to_convert: Vec<SCConversion> }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SCConversion {
    id: u32,
    data: String,
}

pub fn decode(raw_message: String) -> WebSocketMessage {
    let decode_result = serde_json::from_str::<WebSocketMessage>(&raw_message);
    match decode_result {
        Ok(message) => return message,
        Err(err) => WebSocketMessage::Unknown { error: err.to_string() },
    }
}

impl WebSocketMessage {
    pub fn handle(&self) -> WebSocketResponse {
        match self {
            WebSocketMessage::SaveFile { file_path, data, overwrite } => handle_save_file(file_path, data, *overwrite),
            WebSocketMessage::LoadFile { file_path } => todo!(),
            WebSocketMessage::LoadProgram { name, contents } => todo!(),
            WebSocketMessage::RunSC { program_name, to_convert } => todo!(),
            WebSocketMessage::Unknown { error } => WebSocketResponse::Error { message: format!("Unknown message, err: {}", error) },
        } 
    }
}

impl WebSocketResponse {
    pub fn handle(&self) -> Option<String> {
        let temp = serde_json::to_string(self);
        match temp {
            Ok(data) => Some(data),
            Err(_) => None,
        } 
    }
}

fn handle_save_file(file_path: &String, data: &String, overwrite: bool) -> WebSocketResponse {
    let result = save_to_file(file_path, data, overwrite, true);
    match result {
        Some(v) => match v {
            IOError::FileNotFound(err) => WebSocketResponse::Error { message: err },
            IOError::ReadError(err) => WebSocketResponse::Error { message: err },
            IOError::InvalidFilePath(err) => WebSocketResponse::Error { message: err },
            IOError::WriteError(err) => WebSocketResponse::Error { message: err },
            IOError::Other(err) => WebSocketResponse::Error { message: err },
            IOError::FileExists(_) => WebSocketResponse::RequestOverwrite,
        },
        None => WebSocketResponse::Success
    }
}