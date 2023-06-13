use crate::{manual_ux::table, io};

use super::table::Table;

pub struct Project {
    pub tables: Vec<Option<Table>>,
}

pub fn load_project(filepath: String) -> Project {
    let mut max_id: usize = 0;
    let mut accumulation: Vec<Table> = Vec::new();
    for entry in glob::glob(&(filepath + &String::from("/**/*.ltable"))).unwrap() {
        let temp = table::load_table(&io::load_from_file(&entry.unwrap().as_path().display().to_string(), false).unwrap());
        if <u16 as Into<usize>>::into(temp.id) > max_id {
            max_id = temp.id.into();
        }
        accumulation.push(temp);
    }

    let mut tables = Vec::with_capacity(max_id);

    // Initial population
    let mut i = 0;
    while i < max_id {
        tables.push(Option::None);
        i += 1;
    }

    // Population with accumulation
    for table in accumulation {
        let id: usize = table.id.into();
        if tables[id].is_some() {
            panic!("ID Collision")
        }
        tables[id] = Option::Some(table);
    }

    Project { tables }
}   