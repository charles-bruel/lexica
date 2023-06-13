use std::string;

use crate::{manual_ux::table, io};

use super::table::Table;

pub struct Project {
    pub tables: Vec<Option<Table>>,
}

pub fn load_project(filepath: String) -> Project {
    let mut accumulation: Vec<Table> = Vec::new();
    for entry in glob::glob(&(filepath + &String::from("/**/*.ltable"))).unwrap() {
        accumulation.push(table::load_table(&io::load_from_file(&entry.unwrap().as_path().display().to_string(), false).unwrap()));
    }
    println!("{:?}", accumulation);
    todo!()
}   