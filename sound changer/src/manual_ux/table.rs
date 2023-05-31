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
    todo!()
}