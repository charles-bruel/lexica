use std::{collections::HashMap, rc::Rc};

use crate::{io, manual_ux::table, sc::data::Program};

use super::table::{LoadingErrorType, Table, TableDescriptor, TableLoadingError};

pub struct Project {
    pub programs: HashMap<String, Program>,
    pub tables: Vec<Option<Table>>,
}

impl Project {
    pub fn insert_table(&mut self, table: Table) {
        // Inserts a table into the the vector of tables
        // First checks if the vector is big enough to not need expanding
        // If it is, it replaces the relevant None with Some(table)
        // If it isn't, it expands the vector to the required size and then inserts the table
        let id: usize = table.id.into();
        if id < self.tables.len() {
            self.tables[id] = Option::Some(table);
        } else {
            let required_capacity = id + 1;
            let mut i = self.tables.len();
            while i < required_capacity {
                self.tables.push(Option::None);
                i += 1;
            }
            self.tables[id] = Option::Some(table);
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ProjectLoadError {
    TableError(TableLoadingError),
}

pub fn load_project(filepath: String) -> Result<Project, ProjectLoadError> {
    let mut max_id: usize = 0;
    let mut accumulation: Vec<Table> = Vec::new();
    let mut previous_descriptors: HashMap<usize, Rc<TableDescriptor>> = HashMap::new();
    // TODO: Sort table load order so they get loaded in order even if the names are bad
    for entry in glob::glob(&(filepath + &String::from("/**/*.ltable"))).unwrap() {
        let path_str = &entry.unwrap().as_path().display().to_string();
        let temp = match table::load_table(
            &io::load_from_file(path_str, false).unwrap(),
            &mut previous_descriptors,
            path_str.clone(),
        ) {
            Ok(v) => v,
            Err(e) => {
                println!("{:?}", e);
                println!();
                panic!();
            }
        };

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
            return Err(ProjectLoadError::TableError(TableLoadingError {
                error_type: LoadingErrorType::TableIDCollision,
            }));
        }
        tables[id] = Option::Some(table);
    }

    Ok(Project {
        tables,
        programs: HashMap::new(),
    })
}
