use std::rc::Rc;

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
pub struct TableRow {
    pub descriptor: Rc<TableDescriptor>,
    pub contents: Vec<TableContents>,
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
    Enum(u8),
    String(String),
    UInt(u32),
    Int(i32),
}

pub fn load_table(input: &String) -> Table {
    let mut lines: Vec<&str> = input.split("\n").collect();
    let header1 = lines[0].trim();
    let header2 = lines[1].trim();
    let header3 = lines[2].trim();
    lines = lines.drain(0..2).collect();

    let id = header1.parse::<u16>().unwrap();

    let descriptors_names: Vec<&str> = header2.split("|").collect();
    let descriptors_contents: Vec<&str> = header3.split("|").collect();
    assert_eq!(descriptors_names.len(), descriptors_contents.len());

    let mut descriptors: Vec<TableColumnDescriptor> = Vec::new();
    let mut i = 0;
    while i < descriptors_names.len() {
        let name = descriptors_names[i];
        let data_type = load_table_data_type(descriptors_contents[i]);
        
        descriptors.push(TableColumnDescriptor { name: name.to_string(), data_type: data_type });

        i += 1;
    }

    for line in lines {
        
    }

    !todo!()

    // Table {
    //     id,
    //     table_descriptor: descriptors,
    //     table_rows: todo!(),
    // }
}

pub fn load_table_data_type(input: &str) -> TableDataTypeDescriptor {
    // Case non sensitive
    let mut value = String::from(input).to_lowercase();
    // First we check the basic data types
    if value == "string" {
        return TableDataTypeDescriptor::String;
    }
    if value == "int" {
        return TableDataTypeDescriptor::Int;
    }
    if value == "uint" {
        return TableDataTypeDescriptor::UInt;
    }

    // Enums are represented as a bracket-surrounded, comma-seperated list of values.
    if value.starts_with("[") && value.ends_with("]") {
        // It's formatted like an enum

        value = value.trim_start_matches("[").to_string();
        value = value.trim_end_matches("]").to_string();
        
        let enum_values: Vec<&str> = value.split(",").collect();
        let mut enum_final_values: Vec<String> = Vec::new();
        for x in enum_values {
            enum_final_values.push(x.trim().to_string());
        }

        return TableDataTypeDescriptor::Enum(enum_final_values);
    }

    // Todo: Real error handling
    todo!()
}