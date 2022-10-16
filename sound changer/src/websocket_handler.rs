use serde::{Serialize, Deserialize};
use super::data::{ThreadContext, to_string, ApplicationError};
use super::applicator::*;
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SCConversion {
    pub id: u32,
    pub data: Result<String, ApplicationError>,
}

pub fn decode(raw_message: String) -> WebSocketMessage {
    let decode_result = serde_json::from_str::<WebSocketMessage>(&raw_message);
    match decode_result {
        Ok(message) => return message,
        Err(err) => WebSocketMessage::Unknown { error: err.to_string() },
    }
}

impl WebSocketMessage {
    pub fn handle(&self, context: &mut ThreadContext) -> WebSocketResponse {
        match self {
            WebSocketMessage::SaveFile { file_path, data, overwrite } => handle_save_file(file_path, data, *overwrite),
            WebSocketMessage::LoadFile { file_path } => handle_load_file(file_path),
            WebSocketMessage::LoadProgram { name, contents } => todo!(),
            WebSocketMessage::RunSC { program_name, to_convert } => handle_run_sc(program_name, to_convert, context),
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
            IOError::FileExists(_) => WebSocketResponse::RequestOverwrite,
            _ => v.get_response()
        },
        None => WebSocketResponse::Success
    }
}

fn handle_load_file(file_path: &String) -> WebSocketResponse {
    let data = load_from_file(file_path, true);
    match data {
        Ok(v) => WebSocketResponse::LoadFileResult { data: v },
        Err(v) => v.get_response(),
    }
}

fn send_error_response(error: (ApplicationError, usize, String), context: &mut ThreadContext) {
    let response = WebSocketResponse::Error { message: format!("Issue converting word \"{}\" (id: {}), error: {}", error.2, error.1, error.0) };

    context.queued_extra_messages.push_back(response);
}

fn handle_run_sc(program_name: &String, to_convert: &Vec<SCConversion>, context: &mut ThreadContext) -> WebSocketResponse {
    if context.programs.contains_key(program_name) {
        let program = context.programs.get(program_name).unwrap();
        let mut result = (*to_convert).clone();

        let mut errors: Vec<(ApplicationError, usize, String)> = Vec::new();

        let mut i: usize = 0;
        while i < result.len() {
            if !&result[i].data.is_ok() {
                i += 1;
                continue;
            }
            let input = result[i].data.as_ref().unwrap();
            match from_string(&program, input) {
                Ok(val) => {
                    match program.apply(val) {
                        Ok(v) => {
                            let output = to_string(&program, v);
                            result[i].data = output;
                        },
                        Err(v) => { errors.push((v, i, input.clone())); },
                    };
                },
                Err(v) => { errors.push((v, i, input.clone())); },
            };
            
            i += 1;
        }

        for x in errors {
            send_error_response(x, context);
        }

        WebSocketResponse::RunSCResult { to_convert: result }
    } else {
        WebSocketResponse::Error { message: format!("Unknown program name \"{}\"", program_name) }
    }
}