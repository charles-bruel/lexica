use crate::{manual_ux::table, io};

use super::table::{Table, TableLoadingError};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Project {
    pub tables: Vec<Option<Table>>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ProjectLoadError {
    TableError(TableLoadingError),
}

pub fn load_project(filepath: String) -> Result<Project, ProjectLoadError> {
    let mut max_id: usize = 0;
    let mut accumulation: Vec<Table> = Vec::new();
    for entry in glob::glob(&(filepath + &String::from("/**/*.ltable"))).unwrap() {
        let temp = table::load_table(&io::load_from_file(&entry.unwrap().as_path().display().to_string(), false).unwrap()).unwrap();
        if <u16 as Into<usize>>::into(temp.id) > max_id {
            max_id = temp.id.into();
        }
        accumulation.push(temp);
    }

    let required_capacity = max_id + 1;

    let mut tables = Vec::with_capacity(required_capacity);

    // Initial population
    let mut i = 0;
    while i < required_capacity {
        tables.push(Option::None);
        i += 1;
    }

    // Population with accumulation
    for table in accumulation {
        let id: usize = table.id.into();
        if tables[id].is_some() {
            return Err(ProjectLoadError::TableError(TableLoadingError::TableIDCollision));
        }
        tables[id] = Option::Some(table);
    }

    Ok(Project { tables })
}   