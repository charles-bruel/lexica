use crate::manual_ux::rebuilder::rebuild;
use crate::manual_ux::table::{self, Table};
use crate::sc::constructor::construct;
use crate::sc::data::ConstructorError;
use serde::{Deserialize, Serialize};

use super::io::*;
use super::sc::applicator::*;
use super::sc::data::{to_string, ApplicationError, ThreadContext};

#[derive(Deserialize, Debug)]
pub enum WebSocketMessage {
    SaveFile {
        file_path: String,
        data: String,
        overwrite: bool,
    },
    LoadFile {
        file_path: String,
    },
    LoadProgram {
        name: String,
        contents: String,
    },
    TryCompile {
        program: String,
    },
    RunSC {
        program_name: String,
        to_convert: Vec<SCConversion>,
    },
    LoadTable {
        contents: String,
    },
    RebuildTables {
        start_index: u16,
    },
    Unknown {
        error: String,
    },
}

#[derive(Serialize, Debug)]
pub enum WebSocketResponse {
    Success,
    Error { message: String },
    RequestOverwrite,
    LoadFileResult { data: String },
    RunSCResult { to_convert: Vec<SCConversion> },
    CompilationResult { result: Option<ConstructorError> },
    TableResult { table: Option<Table> },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SCConversion {
    pub id: u32,
    pub data: Result<String, ApplicationError>,
}

pub fn decode(raw_message: String) -> WebSocketMessage {
    let decode_result = serde_json::from_str::<WebSocketMessage>(&raw_message);
    match decode_result {
        Ok(message) => message,
        Err(err) => WebSocketMessage::Unknown {
            error: err.to_string(),
        },
    }
}

impl WebSocketMessage {
    pub fn handle(&self, context: &mut ThreadContext) -> Vec<WebSocketResponse> {
        match self {
            WebSocketMessage::SaveFile {
                file_path,
                data,
                overwrite,
            } => vec![handle_save_file(file_path, data, *overwrite)],
            WebSocketMessage::LoadFile { file_path } => vec![handle_load_file(file_path)],
            WebSocketMessage::LoadProgram { name, contents } => {
                vec![handle_load_program(name, contents, context)]
            }
            WebSocketMessage::TryCompile { program } => vec![handle_try_compilation(program)],
            WebSocketMessage::RunSC {
                program_name,
                to_convert,
            } => vec![handle_run_sc(program_name, to_convert, context)],
            WebSocketMessage::Unknown { error } => vec![WebSocketResponse::Error {
                message: format!("Unknown message, err: {}", error),
            }],
            WebSocketMessage::LoadTable { contents } => {
                let mut previous_descriptors = context
                    .project
                    .tables
                    .iter()
                    .filter_map(|p| {
                        p.as_ref()
                            .map(|v| (v.id as usize, v.table_descriptor.clone()))
                    })
                    .collect();
                match table::load_table(
                    contents.as_str(),
                    &mut previous_descriptors,
                    String::from("NOT GIVEN"),
                ) {
                    Ok(v) => {
                        context.project.insert_table(v.clone());
                        vec![WebSocketResponse::TableResult { table: Some(v) }]
                    }
                    Err(_) => vec![WebSocketResponse::TableResult { table: None }],
                }
            }
            WebSocketMessage::RebuildTables { start_index } => {
                rebuild(
                    &mut context.project,
                    *start_index,
                    String::from("TODO"),
                    false,
                );

                // Send all tables from after the start index
                let mut result = Vec::new();
                for table in context
                    .project
                    .tables
                    .iter()
                    .skip(*start_index as usize)
                    .flatten()
                {
                    result.push(WebSocketResponse::TableResult {
                        table: Some(table.clone()),
                    });
                }
                result
            }
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
            _ => v.get_response(),
        },
        None => WebSocketResponse::Success,
    }
}

fn handle_load_file(file_path: &String) -> WebSocketResponse {
    let data = load_from_file(file_path, true);
    match data {
        Ok(v) => WebSocketResponse::LoadFileResult { data: v },
        Err(v) => v.get_response(),
    }
}

fn handle_load_program(
    name: &String,
    contents: &str,
    context: &mut ThreadContext,
) -> WebSocketResponse {
    let program = construct(contents);
    match program {
        Ok(v) => {
            if context.project.programs.contains_key(name) {
                context.project.programs.remove(name);
            }
            context.project.programs.insert(name.to_string(), v);
            WebSocketResponse::Success
        }
        Err(_) => WebSocketResponse::Error {
            message: format!("Error compiling program \"{}\"", name),
        }, //No error message because that should already be there
    }
}

fn handle_try_compilation(program: &str) -> WebSocketResponse {
    let result = construct(program);
    WebSocketResponse::CompilationResult {
        result: match result {
            Ok(_) => None,
            Err(v) => Some(v),
        },
    }
}

fn send_error_response(error: (ApplicationError, usize, String), context: &mut ThreadContext) {
    let response = WebSocketResponse::Error {
        message: format!(
            "Issue converting word \"{}\" (id: {}), error: {}",
            error.2, error.1, error.0
        ),
    };

    context.queued_extra_messages.push_back(response);
}

fn handle_run_sc(
    program_name: &String,
    to_convert: &Vec<SCConversion>,
    context: &mut ThreadContext,
) -> WebSocketResponse {
    if context.project.programs.contains_key(program_name) {
        let program = context.project.programs.get(program_name).unwrap();
        let mut result = (*to_convert).clone();

        let mut errors: Vec<(ApplicationError, usize, String)> = Vec::new();

        let mut i: usize = 0;
        while i < result.len() {
            if !&result[i].data.is_ok() {
                i += 1;
                continue;
            }
            let input = result[i].data.as_ref().unwrap();
            match from_string(program, input) {
                Ok(val) => {
                    match program.apply(val) {
                        Ok(v) => {
                            let output = to_string(program, v);
                            result[i].data = output;
                        }
                        Err(v) => {
                            errors.push((v, i, input.clone()));
                        }
                    };
                }
                Err(v) => {
                    errors.push((v, i, input.clone()));
                }
            };

            i += 1;
        }

        for x in errors {
            send_error_response(x, context);
        }

        WebSocketResponse::RunSCResult { to_convert: result }
    } else {
        WebSocketResponse::Error {
            message: format!("Unknown program name \"{}\"", program_name),
        }
    }
}
