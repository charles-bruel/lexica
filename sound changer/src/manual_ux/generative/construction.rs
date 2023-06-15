use std::rc::Rc;

use crate::manual_ux::{project::Project, table::{TableDescriptor, TableLoadingError, TableRow}};

use super::{tokenizer::{Token, self, tokenize}, GenerativeProgram};

pub fn parse_generative_table_line(descriptor: &TableDescriptor, line: &str) -> Result<TableRow, TableLoadingError> {
    let tokens = tokenize(line.to_string());
    
    todo!()
}