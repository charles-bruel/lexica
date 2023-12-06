use std::{collections::HashMap, rc::Rc};

use crate::rand::Rng;

use crate::{
    io,
    manual_ux::{
        conjugator::{create_conjugations, ConjugatorInput},
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
            tables: vec![None, Some(table)],
            programs: HashMap::new()
        })
    );
}

fn int_test(path_str: &str) {
    let path = String::from(path_str);
    let mut project = load_project(path.clone()).unwrap();
    rebuild(&mut project, 0, path.clone(), true);
    let comp = io::load_from_file(&(path + "/correct.txt"), false)
        .unwrap()
        .replace("\r\n", "\n");

    assert_eq!(
        comp,
        project.tables[project.tables.len() - 1]
            .clone()
            .unwrap()
            .output(&project)
    )
}

#[test]
fn test_int_1() {
    const PATH_STR: &str = "test-data/backend/1";
    int_test(PATH_STR);
}

#[test]
fn test_int_2() {
    const PATH_STR: &str = "test-data/backend/2";
    int_test(PATH_STR);
}

#[test]
fn test_int_3() {
    const PATH_STR: &str = "test-data/backend/3";
    int_test(PATH_STR);
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
                    column: ColumnSpecifier { column_id: 0 },
                    name: String::from("a1"),
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

#[test]
fn conjugator_indep_int_1() {
    let contents = io::load_from_file(
        &String::from("test-data/backend/conjugator_test_data_1.txt"),
        false,
    )
    .unwrap();
    let lines: Vec<&str> = contents.lines().collect();
    let mut words = vec![vec![]];
    for line in lines {
        // Fill the vector in words until there are 8 elements in it, then move onto the next vector
        if words[words.len() - 1].len() == 8 {
            words.push(vec![]);
        }
        let idx = words.len() - 1;
        words[idx].push(String::from(line));
    }

    let input = ConjugatorInput {
        words,
        max_conjugations: 1,
        max_intraconjugation_roots: 1,
        max_alternations: 3,
    };

    create_conjugations(input);
}

#[test]
fn conjugator_indep_int_2() {
    let contents = io::load_from_file(
        &String::from("test-data/backend/conjugator_test_data_2.txt"),
        false,
    )
    .unwrap();
    let lines: Vec<&str> = contents.lines().collect();
    let mut words = vec![vec![]];
    for line in lines {
        // Fill the vector in words until there are 8 elements in it, then move onto the next vector
        if words[words.len() - 1].len() == 8 {
            words.push(vec![]);
        }
        let idx = words.len() - 1;
        words[idx].push(String::from(line));
    }

    let input = ConjugatorInput {
        words,
        max_conjugations: 1,
        max_intraconjugation_roots: 1,
        max_alternations: 10,
    };

    let out = create_conjugations(input);

    assert_eq!(out.conjugations.len(), 1);
}

#[test]
fn conjugator_indep_int_3() {
    let contents = io::load_from_file(
        &String::from("test-data/backend/conjugator_test_data_3.txt"),
        false,
    )
    .unwrap();
    let lines: Vec<&str> = contents.lines().collect();
    let mut words = vec![vec![]];
    for line in lines {
        // Fill the vector in words until there are 2 elements in it, then move onto the next vector
        if words[words.len() - 1].len() == 2 {
            words.push(vec![]);
        }
        let idx = words.len() - 1;
        words[idx].push(String::from(line));
    }

    let input = ConjugatorInput {
        words,
        max_conjugations: 1,
        max_intraconjugation_roots: 1,
        max_alternations: 3,
    };

    create_conjugations(input);
}

#[test]
fn conjugator_fuzz_1() {
    let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(2);
    for _ in 0..100 {
        let mut words = vec![];
        let affices_per_word = rng.gen_range(5..10);
        for _ in 0..100 {
            let mut word = String::new();
            let mut forms = Vec::new();
            for _ in 0..rng.gen_range(3..8) {
                word.push(rng.gen_range(b'a'..=b'z') as char);
            }
            for _ in 0..affices_per_word {
                let mut form = word.clone();
                for _ in 0..rng.gen_range(1..3) {
                    form.push(rng.gen_range(b'a'..=b'z') as char);
                }
                forms.push(form);
            }

            words.push(forms);
        }

        let input = ConjugatorInput {
            words,
            max_conjugations: 100,
            max_intraconjugation_roots: 6,
            max_alternations: 6,
        };
        _ = create_conjugations(input);
    }
}
/*#[test]
fn table_generative_load_correct_1() {
    const TEST_HEADER: &str = "0\na\nstring\n:={foo}";
    let mut empty_hash: HashMap<usize, Rc<TableDescriptor>> = HashMap::new();
    let result = load_table(TEST_HEADER, &mut empty_hash, String::from("")).unwrap();
    match result.table_rows[0].clone() {
        TableRow::PopulatedTableRow {
            source,
            descriptor: _,
            contents,
        } => {
            assert_eq!(source, PopulatedTableRowSource::EXPLICIT);
            assert_eq!(contents[0], TableContents::String(String::from("test")));
        }
        _ => panic!(),
    }
}

#[test]
fn table_generative_load_correct_2() {
    const TEST_HEADER: &str = "0\na\nint\n:={1}";
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
*/
