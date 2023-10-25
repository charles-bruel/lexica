use std::{collections::HashMap, rc::Rc};

use crate::{
    io,
    manual_ux::{
        project::{load_project, Project},
        table::{load_table, TableDescriptor},
    },
};

use super::rebuilder::rebuild;

#[test]
fn test_load_table_1() {
    const PATH_STR: &str = "test-data/backend/1/001.ltable";
    let mut hash: HashMap<usize, Rc<TableDescriptor>> = HashMap::new();
    let path = String::from(PATH_STR);
    load_table(
        &io::load_from_file(&path, false).unwrap(),
        &mut hash,
        path.clone(),
    )
    .unwrap();
}

#[test]
fn test_load_table_2() {
    const PATH_STR_1: &str = "test-data/backend/1/001.ltable";
    const PATH_STR_2: &str = "test-data/backend/1/002.ltable";
    let mut hash: HashMap<usize, Rc<TableDescriptor>> = HashMap::new();
    let mut path = String::from(PATH_STR_1);
    load_table(
        &io::load_from_file(&path, false).unwrap(),
        &mut hash,
        path.clone(),
    )
    .unwrap();
    path = String::from(PATH_STR_2);
    load_table(
        &io::load_from_file(&path, false).unwrap(),
        &mut hash,
        path.clone(),
    )
    .unwrap();
}

#[test]
#[should_panic]
fn test_load_table_3() {
    const PATH_STR: &str = "test-data/backend/1/002.ltable";
    let mut hash: HashMap<usize, Rc<TableDescriptor>> = HashMap::new();
    let path = String::from(PATH_STR);
    load_table(
        &io::load_from_file(&path, false).unwrap(),
        &mut hash,
        path.clone(),
    )
    .unwrap();
}

#[test]
fn test_load_project_1() {
    const PATH_STR: &str = "test-data/backend/1";
    load_project(String::from(PATH_STR)).unwrap();
}

#[test]
fn test_load_project_2() {
    const PATH_STR: &str = "test-data/backend/2";
    load_project(String::from(PATH_STR)).unwrap();
}

#[test]
fn test_table_output_fn() {
    const PATH_STR: &str = "test-data/backend/1/001.ltable";
    let mut hash: HashMap<usize, Rc<TableDescriptor>> = HashMap::new();
    let path = String::from(PATH_STR);
    let table = load_table(
        &io::load_from_file(&path, false).unwrap(),
        &mut hash,
        path.clone(),
    )
    .unwrap();

    const COMP: &str = "| POS       | WORD  | TRANSLATION | INDEX |
|-----------|-------|-------------|-------|
| root      | ran   | earth       | 0     |
| noun      | kosa  | sword       | 1     |
| pronoun   | ipi   | I           | 2     |
| adjective | von   | red         | 3     |
| adverb    | rele  | quickly     | 4     |
| particle  | na    | nominalizer | 5     |
| noun      | kasii | lightning   | 6     |
| noun      | eʃa   | king        | 7     |";

    assert_eq!(
        COMP,
        table.clone().output(&Project {
            tables: vec![None, Some(table)]
        })
    );
}

#[test]
fn test_int_1() {
    const PATH_STR: &str = "test-data/backend/1";
    let path = String::from(PATH_STR);
    let mut project = load_project(path.clone()).unwrap();
    rebuild(&mut project, 0, path, true);
    const COMP: &str = "| POS       | WORD     | TRANSLATION | INDEX |
|-----------|----------|-------------|-------|
| noun      | gosajka  | sword       | 1     |
| noun      | kasʲiːka | lightning   | 6     |
| noun      | eʃajka   | king        | 7     |
| root      | an       | earth       | 0     |
| pronoun   | ipi      | I           | 2     |
| adjective | von      | red         | 3     |
| adverb    | eː       | quickly     | 4     |";

    assert_eq!(COMP, project.tables[2].clone().unwrap().output(&project))
}
