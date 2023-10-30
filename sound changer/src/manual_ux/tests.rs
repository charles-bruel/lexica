use std::{collections::HashMap, rc::Rc};

use crate::{
    io,
    manual_ux::{
        generative::{
            execution::{ColumnSpecifier, RuntimeEnum, TableSpecifier},
            CompileAttribution, CompileErrorType, GenerativeProgramCompileError, SyntaxErrorType,
        },
        project::{load_project, Project},
        table::{
            load_table, LoadingErrorType, PopulatedTableRowSource, TableContents, TableDescriptor,
            TableRow,
        },
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

#[test]
fn table_header_test_1() {
    const TEST_HEADER: &str = "";
    let mut empty_hash: HashMap<usize, Rc<TableDescriptor>> = HashMap::new();
    let result = load_table(TEST_HEADER, &mut empty_hash, String::from(""));
    assert_eq!(
        result.unwrap_err().error_type,
        LoadingErrorType::MalformedHeader
    );
}

#[test]
fn table_header_test_2() {
    const TEST_HEADER: &str = "-1\nfoo|\nint";
    let mut empty_hash: HashMap<usize, Rc<TableDescriptor>> = HashMap::new();
    let result = load_table(TEST_HEADER, &mut empty_hash, String::from(""));
    assert_eq!(
        result.unwrap_err().error_type,
        LoadingErrorType::MalformedHeader
    );
}

#[test]
fn table_header_test_3() {
    const TEST_HEADER: &str = "1\nfoo\nint";
    let mut empty_hash: HashMap<usize, Rc<TableDescriptor>> = HashMap::new();
    let result = load_table(TEST_HEADER, &mut empty_hash, String::from(""));
    assert!(result.is_ok())
}

#[test]
fn table_header_test_4() {
    const TEST_HEADER: &str = "0\nfoo\nasdasdsa";
    let mut empty_hash: HashMap<usize, Rc<TableDescriptor>> = HashMap::new();
    let result = load_table(TEST_HEADER, &mut empty_hash, String::from(""));
    assert_eq!(
        result.unwrap_err().error_type,
        LoadingErrorType::MalformedDataTypeDescriptor
    );
}

#[test]
fn table_header_test_5() {
    const TEST_HEADER: &str = "0\nfoo\nint|int";
    let mut empty_hash: HashMap<usize, Rc<TableDescriptor>> = HashMap::new();
    let result = load_table(TEST_HEADER, &mut empty_hash, String::from(""));
    assert_eq!(
        result.unwrap_err().error_type,
        LoadingErrorType::MalformedHeader
    );
}

#[test]
fn table_header_test_6() {
    const TEST_HEADER: &str = "0\nfoo|bar\nint";
    let mut empty_hash: HashMap<usize, Rc<TableDescriptor>> = HashMap::new();
    let result = load_table(TEST_HEADER, &mut empty_hash, String::from(""));
    assert_eq!(
        result.unwrap_err().error_type,
        LoadingErrorType::MalformedHeader
    );
}

#[test]
fn table_type_int_error_1() {
    const TEST_HEADER: &str = "0\nfoo\nint\nbar";
    let mut empty_hash: HashMap<usize, Rc<TableDescriptor>> = HashMap::new();
    let result = load_table(TEST_HEADER, &mut empty_hash, String::from(""));
    assert_eq!(
        result.unwrap_err().error_type,
        LoadingErrorType::ValueParseError
    );
}

#[test]
fn table_type_uint_error_1() {
    const TEST_HEADER: &str = "0\nfoo\nuint\nbar";
    let mut empty_hash: HashMap<usize, Rc<TableDescriptor>> = HashMap::new();
    let result = load_table(TEST_HEADER, &mut empty_hash, String::from(""));
    assert_eq!(
        result.unwrap_err().error_type,
        LoadingErrorType::ValueParseError
    );
}

#[test]
fn table_type_uint_error_2() {
    const TEST_HEADER: &str = "0\nfoo\nuint\n-1";
    let mut empty_hash: HashMap<usize, Rc<TableDescriptor>> = HashMap::new();
    let result = load_table(TEST_HEADER, &mut empty_hash, String::from(""));
    assert_eq!(
        result.unwrap_err().error_type,
        LoadingErrorType::ValueParseError
    );
}

#[test]
fn table_type_enum_error_1() {
    const TEST_HEADER: &str = "0\nfoo\n[bar,baz]\n-1";
    let mut empty_hash: HashMap<usize, Rc<TableDescriptor>> = HashMap::new();
    let result = load_table(TEST_HEADER, &mut empty_hash, String::from(""));
    assert_eq!(
        result.unwrap_err().error_type,
        LoadingErrorType::UnknownEnumType
    );
}

#[test]
fn table_type_load_correct() {
    const TEST_HEADER: &str = "0\na|b|c|d\n[a1,a2]|string|int|uint\na1|test|-1|1";
    let mut empty_hash: HashMap<usize, Rc<TableDescriptor>> = HashMap::new();
    let result = load_table(TEST_HEADER, &mut empty_hash, String::from("")).unwrap();
    match result.table_rows[0].clone() {
        TableRow::PopulatedTableRow {
            source,
            descriptor: _,
            contents,
        } => {
            assert_eq!(source, PopulatedTableRowSource::EXPLICIT);
            assert_eq!(
                contents[0],
                TableContents::Enum(RuntimeEnum {
                    index: 0,
                    table: TableSpecifier { table_id: 0 },
                    column: ColumnSpecifier { column_id: 0 }
                })
            );
            assert_eq!(contents[1], TableContents::String(String::from("test")));
            assert_eq!(contents[2], TableContents::Int(-1));
            assert_eq!(contents[3], TableContents::UInt(1));
        }
        _ => panic!(),
    }
}

#[test]
fn table_generative_load_error_1() {
    const TEST_HEADER: &str = "0\na\nint\n:={}";
    let mut empty_hash: HashMap<usize, Rc<TableDescriptor>> = HashMap::new();
    let result = load_table(TEST_HEADER, &mut empty_hash, String::from(""));
    assert_eq!(
        result.unwrap_err().error_type,
        LoadingErrorType::GenerativeProgramCompileError(GenerativeProgramCompileError {
            error_type: CompileErrorType::SyntaxError(SyntaxErrorType::NoGenerativeContent),
            attribution: CompileAttribution::None
        })
    );
}

#[test]
fn table_generative_load_error_2() {
    const TEST_HEADER: &str = "0\na\nint\n:={ }";
    let mut empty_hash: HashMap<usize, Rc<TableDescriptor>> = HashMap::new();
    let result = load_table(TEST_HEADER, &mut empty_hash, String::from(""));
    assert_eq!(
        result.unwrap_err().error_type,
        LoadingErrorType::GenerativeProgramCompileError(GenerativeProgramCompileError {
            error_type: CompileErrorType::NoValueFromSegment,
            attribution: CompileAttribution::None
        })
    );
}

#[test]
fn table_generative_load_error_3() {
    const TEST_HEADER: &str = "0\na\nint\n:={";
    let mut empty_hash: HashMap<usize, Rc<TableDescriptor>> = HashMap::new();
    let result = load_table(TEST_HEADER, &mut empty_hash, String::from(""));
    assert_eq!(
        result.unwrap_err().error_type,
        LoadingErrorType::GenerativeProgramCompileError(GenerativeProgramCompileError {
            error_type: CompileErrorType::SyntaxError(SyntaxErrorType::MissingProgramSurrondings),
            attribution: CompileAttribution::None
        })
    );
}

#[test]
fn table_generative_load_error_4() {
    const TEST_HEADER: &str = "0\na\nint\n:=";
    let mut empty_hash: HashMap<usize, Rc<TableDescriptor>> = HashMap::new();
    let result = load_table(TEST_HEADER, &mut empty_hash, String::from(""));
    assert_eq!(
        result.unwrap_err().error_type,
        LoadingErrorType::GenerativeProgramCompileError(GenerativeProgramCompileError {
            error_type: CompileErrorType::SyntaxError(SyntaxErrorType::MissingProgramSurrondings),
            attribution: CompileAttribution::None
        })
    );
}
