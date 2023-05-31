use super::table::Table;

pub struct Project {
    pub tables: Vec<Option<Table>>,
}

pub fn load_project(filepath: String) -> Project {
    todo!()
}