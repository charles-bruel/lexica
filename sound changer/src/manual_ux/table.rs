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
    pub name: &str,
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
    let header1 = lines[0];
    let header2 = lines[1];
    let header3 = lines[2];
    lines = lines.drain(0..2);

    let id = header1.parse::<u16>().unwrap();

    let descriptors_names: Vec<&str> = header2.split("|").collect();
    let descriptors_contents: Vec<&str> = header3.split("|").collect();
    assert_eq!(descriptors_names.len(), descriptors_contents.len());

    let mut descriptors: Vec<TableColumnDescriptor> = Vec::new();
    let mut i = 0;
    while i < descriptors_names.len() {
        let name = descriptors_names[i];
        let contents = load_table_data_type(descriptors_contents[i]);
        
        data_type.push(TableColumnDescriptor { name, data_type });

        i += 1;
    }

    for line in lines {
        
    }

    Table {
        id,
        table_descriptor: descriptors,
        table_rows: todo!(),
    }
}

pub fn load_table_data_type(input: &str) {
    todo!()
}