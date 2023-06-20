use std::rc::Rc;

use crate::manual_ux::generative::parse_generative_table_line;

use super::generative::{GenerativeProgramRuntimeError, GenerativeProgramCompileError};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Table {
    pub id: u16,
    pub table_descriptor: Rc<TableDescriptor>,
    pub table_rows: Vec<TableRow>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TableDescriptor {
    pub column_descriptors: Vec<TableColumnDescriptor>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TableRow {
    PopulatedTableRow {
        source: PopulatedTableRowSource,
        descriptor: Rc<TableDescriptor>,
        contents: Vec<TableContents>,
    },
    UnpopulatedTableRow {
        procedure: Rc<GenerativeTableRowProcedure>,
        descriptor: Rc<TableDescriptor>,
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PopulatedTableRowSource {
    EXPLICIT, CACHE(Rc<GenerativeTableRowProcedure>)
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GenerativeTableRowProcedure {
    // TODO
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TableColumnDescriptor {
    pub name: String,
    pub data_type: TableDataTypeDescriptor,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TableDataTypeDescriptor {
    Enum(Vec<String>),
    String,
    UInt,
    Int,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TableContents {
    Enum(usize),
    String(String),
    UInt(u32),
    Int(i32),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TableLoadingError {
    MalformedHeader,
    MalformedDataTypeDescriptor,
    UnknownEnumType,
    ValueParseError,
    TableIDCollision,
    GenerativeProgramError(GenerativeProgramCompileError),

    Unknown,
}

pub fn load_table(input: &String) -> Result<Table, TableLoadingError> {
    let mut lines: Vec<&str> = input.split("\n").collect();
    if lines.len() < 3 {
        return Err(TableLoadingError::MalformedHeader)
    }
    let header1 = lines[0].trim();
    let header2 = lines[1].trim();
    let header3 = lines[2].trim();
    lines = lines.drain(3..).collect();

    let id = match header1.parse::<u16>() {
        Ok(v) => v,
        Err(_) => return Err(TableLoadingError::MalformedHeader),
    };

    let descriptors_names: Vec<&str> = header2.split("|").collect();
    let descriptors_contents: Vec<&str> = header3.split("|").collect();
    if descriptors_names.len() != descriptors_contents.len() {
        return Err(TableLoadingError::MalformedHeader)
    }

    let mut descriptors: Vec<TableColumnDescriptor> = Vec::new();
    let mut i = 0;
    while i < descriptors_names.len() {
        let name = descriptors_names[i];
        let data_type = load_table_data_type(descriptors_contents[i])?;
        
        descriptors.push(TableColumnDescriptor { name: name.to_string(), data_type: data_type });

        i += 1;
    }

    let descriptor = TableDescriptor { column_descriptors: descriptors };

    let mut table_rows: Vec<TableRow> = Vec::new();

    let table_descriptor = Rc::new(descriptor);

    for line in lines {
        table_rows.push(parse_table_line(table_descriptor.clone(), line)?);
    }

    Ok(Table {
        id,
        table_descriptor,
        table_rows,
    })
}

fn load_table_data_type(input: &str) -> Result<TableDataTypeDescriptor, TableLoadingError> {
    // Case non sensitive
    let mut value = String::from(input).to_lowercase();
    // First we check the basic data types
    if value == "string" {
        return Ok(TableDataTypeDescriptor::String);
    }
    if value == "int" {
        return Ok(TableDataTypeDescriptor::Int);
    }
    if value == "uint" {
        return Ok(TableDataTypeDescriptor::UInt);
    }

    // Enums are represented as a bracket-surrounded, comma-seperated list of values.
    if value.starts_with("[") && value.ends_with("]") {
        // It's formatted like an enum

        value = value.trim_start_matches("[").to_string();
        value = value.trim_end_matches("]").to_string();
        
        let enum_values: Vec<&str> = value.split(",").collect();
        let mut enum_final_values: Vec<String> = Vec::new();
        for x in enum_values {
            enum_final_values.push(x.to_lowercase().trim().to_string());
        }

        return Ok(TableDataTypeDescriptor::Enum(enum_final_values));
        
    }

    Err(TableLoadingError::MalformedDataTypeDescriptor)
}

fn parse_table_line(descriptor: Rc<TableDescriptor>, line: &str) -> Result<TableRow, TableLoadingError> {
    if line.starts_with(":=") {
        return parse_generative_table_line(descriptor.as_ref(), line);
    }

    let values: Vec<&str> = line.split("|").collect();
    assert_eq!(values.len(), descriptor.column_descriptors.len());

    let mut i = 0;
    let mut cells: Vec<TableContents> = Vec::new();
    while i < values.len() {
        cells.push(parse_table_cell(values[i], &descriptor.as_ref().column_descriptors[i].data_type)?);

        i += 1;
    }

    Ok(TableRow::PopulatedTableRow { source: PopulatedTableRowSource::EXPLICIT, descriptor: descriptor, contents: cells })
}

fn parse_table_cell(cell_contents: &str, descriptor: &TableDataTypeDescriptor) -> Result<TableContents, TableLoadingError> {
    match descriptor {
        TableDataTypeDescriptor::Enum(vec) => {
            let test_string = cell_contents.to_lowercase();
            // I'm sure there is an API way to do this, but I
            // am on a plane right now and can't find it, so
            // I'm doing it manually

            let mut i = 0;
            while i < vec.len() {
                if test_string == vec[i] {
                    return Ok(TableContents::Enum(i));
                }

                i += 1;
            }

            Err(TableLoadingError::UnknownEnumType)
        },
        TableDataTypeDescriptor::String => {
            Ok(TableContents::String(cell_contents.to_string()))
        },
        TableDataTypeDescriptor::UInt => {
            match cell_contents.parse::<u32>() { 
                Ok(v) => Ok(TableContents::UInt(v)),
                Err(_) => Err(TableLoadingError::ValueParseError),
            }
        },
        TableDataTypeDescriptor::Int => {
            match cell_contents.parse::<i32>() { 
                Ok(v) => Ok(TableContents::Int(v)),
                Err(_) => Err(TableLoadingError::ValueParseError),
            }
        },
    }
}