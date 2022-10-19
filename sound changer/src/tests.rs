use super::data::*;
use super::rules::*;
use super::constructor::*;
use super::applicator::*;
use super::io::*;

extern crate rand;
use self::rand::Rng;

#[test]
fn test_simple_result() {
    let letter = random_letter();
    let result_rule = create_simple_result(letter);
    let result = result_rule.transform(&random_letter()).unwrap();
    assert_eq!(letter, result)
}

#[test]
fn test_delete_result() {
    let result_rule = create_delete_result();
    let result = result_rule.transform(&random_letter());
    assert_eq!(result, None)
}

#[test]
fn test_application_result() {
    let mask = random_u64();
    let key = random_u64() & mask;
    let input = random_letter();
    let result_rule = create_simple_application_result(mask, key);
    let result = result_rule.transform(&input).unwrap();
    assert_eq!((input.value & !mask) | key, result.value)
}

#[test]
fn test_simple_predicate_a() {
    let mask: u64 = 0xFFFFFFFFFFFFFFFF;
    let key = random_u64();
    let pos_test = super::data::Letter{ value: key };
    let neg_test = loop {
        let neg = random_letter();
        if neg.value != key { break neg }
    };

    let predicate = create_simple_predicate(key, mask);

    assert!(predicate.validate(&vec!(pos_test), 0));
    assert!(!predicate.validate(&vec!(neg_test), 0));
}

#[test]
fn test_simple_predicate_b() {
    let mask: u64 = random_u64();
    let key = mask & random_u64();
    let pos_test = super::data::Letter{ value: key };
    let neg_test = loop {
        let neg = random_letter();
        if neg.value != key { break neg }
    };

    let predicate = create_simple_predicate(key, mask);

    assert!(predicate.validate(&vec!(pos_test), 0));
    assert!(!predicate.validate(&vec!(neg_test), 0));
}

#[test]
fn test_multi_predicate_a() {
    let key = random_u64();
    let mut tests: Vec<Box<dyn Predicate>> = Vec::new();

    let mut i: usize = 0;
    while i < 64 {
        let mask: u64 = 1 << i;
        let current_key = key & mask;
        let predicate = create_simple_predicate(current_key, mask);
        tests.push(Box::new(predicate));

        i += 1;
    }

    let pos_test = super::data::Letter{ value: key };
    let neg_test = loop {
        let neg = random_letter();
        if neg.value != key { break neg }
    };

    let predicate = create_multi_predicate(tests, true);

    assert!(predicate.validate(&vec!(pos_test), 0));
    assert!(!predicate.validate(&vec!(neg_test), 0));
}

#[test]
fn test_multi_predicate_b() {
    let key = random_u64();
    let mut tests: Vec<Box<dyn Predicate>> = Vec::new();

    let mut i: usize = 0;
    while i < 64 {
        let mask: u64 = 1 << i;
        let current_key = key & mask;
        let predicate = create_simple_predicate(current_key, mask);
        tests.push(Box::new(predicate));

        i += 1;
    }

    let fudge = loop {
        let temp = random_u64();
        if temp != 0 { break temp }
    };

    let pos_test = super::data::Letter{ value: key & fudge };
    let neg_test = super::data::Letter{ value: !key };

    let predicate = create_multi_predicate(tests, false);

    assert!(predicate.validate(&vec!(pos_test), 0));
    assert!(!predicate.validate(&vec!(neg_test), 0));
}

#[test]
fn test_positive_negative_predicate() {
    let mask: u64 = random_u64();
    let key = random_u64();

    let neg_mask = loop {
        let temp = random_u64() & !mask;
        if temp != 0 { break temp }
    };
    let neg_key = !key & neg_mask;

    let pos_test = super::data::Letter{ value: key };
    let neg_test_1 = super::data::Letter{ value: (key & !neg_mask) | !(key & neg_mask) };
    let neg_test_2 = loop {
        let neg = random_letter();
        if neg.value & mask != key { break neg }
    };

    let predicate = create_positive_negative_predicate(mask, key & mask, vec![neg_mask], vec![neg_key]);

    assert!(predicate.validate(&vec!(pos_test), 0));
    assert!(!predicate.validate(&vec!(neg_test_1), 0));
    assert!(!predicate.validate(&vec!(neg_test_2), 0));
}

#[test]
fn test_letter_creation() {
    let program = create_diacritic_test_program();
    let letter = from_string(&program, &String::from("1")).unwrap()[0];
    assert_eq!("1", letter.get_symbol(&program).unwrap());
}

#[test]
fn test_diacritics_a() {
    let program = create_diacritic_test_program();
    let (_, key) = parse_features(&program, "[A1 B1 C1 +toggleA]");
    let letter = Letter { value: key };

    assert_eq!("1ᵃ", letter.get_symbol(&program).unwrap());
}

#[test]
fn test_diacritics_b() {
    let program = create_diacritic_test_program();
    let (_, key) = parse_features(&program, "[A2 B1 C1]");
    let letter = Letter { value: key };

    assert_eq!("1a", letter.get_symbol(&program).unwrap());
}

#[test]
fn test_diacritics_c() {
    let program = create_diacritic_test_program();
    let (_, key) = parse_features(&program, "[A3 B1 C1]");
    let letter = Letter { value: key };

    assert_eq!("1aA", letter.get_symbol(&program).unwrap());
}

#[test]
fn test_diacritics_d() {
    let program = create_diacritic_test_program();
    let (_, key) = parse_features(&program, "[A3 B1 C1 +toggleA]");
    let letter = Letter { value: key };

    assert!(is_anagram(String::from("1aAᵃ"), letter.get_symbol(&program).unwrap()));
}

#[test]
fn test_diacritics_e() {
    let program = create_diacritic_test_program();
    let (_, key) = parse_features(&program, "[A3 B1 C1 +toggleZ]");
    let letter = Letter { value: key };

    assert!(is_anagram(String::from("1aAᶻ"), letter.get_symbol(&program).unwrap()));
}

#[test]
fn test_diacritics_f() {
    let program = create_diacritic_test_program();
    let (_, key) = parse_features(&program, "[A3 B3 C3]");
    let letter = Letter { value: key };

    assert!(is_anagram(String::from("1aAbBcC"), letter.get_symbol(&program).unwrap()));
}

#[test]
fn test_diacritics_g() {
    let program = create_diacritic_test_program();
    let (_, key) = parse_features(&program, "[A3 B3 C3 +toggleA +toggleB +toggleC]");
    let letter = Letter { value: key };

    assert!(is_anagram(String::from("1aAbBcCᵃᵇᶜ"), letter.get_symbol(&program).unwrap()));
}

#[test]
fn test_0_feature_a() {
    let program = create_diacritic_test_program();
    let predicate = construct_simple_predicate(&program, "[A1]");
    let letter: Letter = Letter { value: 0 };
    assert!(!predicate.validate(&vec![letter], 0));
}

#[test]
fn test_0_feature_b() {
    let program = create_diacritic_test_program();
    let predicate = construct_simple_predicate(&program, "[B1]");
    let letter: Letter = Letter { value: 0 };
    assert!(!predicate.validate(&vec![letter], 0));
}

#[test]
fn test_0_feature_c() {
    let program = create_diacritic_test_program();
    let predicate = construct_simple_predicate(&program, "[-toggleA]");
    let letter: Letter = Letter { value: 0 };
    assert!(predicate.validate(&vec![letter], 0));
}

#[test]
fn test_0_feature_d() {
    let program = create_diacritic_test_program();
    let predicate = construct_simple_predicate(&program, "[+toggleA]");
    let letter: Letter = Letter { value: 0 };
    assert!(!predicate.validate(&vec![letter], 0));
}

#[test]
fn test_int_1() {
    let program = create_int_test_1();
    let words = load_from_file(&String::from("test-data/int-test-1.words.txt"), false).unwrap();
    let lines: Vec<&str> = words.split("\n").collect();
    for l in lines {
        let parts: Vec<&str> = l.split(":").collect();
        let word = from_string(&program, &String::from(parts[0].trim())).unwrap();
        let result = to_string(&program, program.apply(word).unwrap());
        assert_eq!(result.unwrap(), parts[1].trim());
    }
}

#[test]
fn test_restrict_path_1() {
    let result = load_from_file(&String::from("C:/foo"), true);
    match result {
        Ok(_) => assert!(false),
        Err(v) => assert!(matches!(v, IOError::InvalidFilePath(_))),
    } 
}

#[test]
fn test_restrict_path_2() {
    let result = load_from_file(&String::from("../foo"), true);
    match result {
        Ok(_) => assert!(false),
        Err(v) => assert!(matches!(v, IOError::InvalidFilePath(_))),
    } 
}

#[test]
fn test_restrict_path_3() {
    let result = save_to_file(&String::from("C:/foo"), &String::from(""), false, true);
    match result {
        None => assert!(false),
        Some(v) => assert!(matches!(v, IOError::InvalidFilePath(_))),
    } 
}

#[test]
fn test_restrict_path_4() {
    let result = save_to_file(&String::from("../foo"), &String::from(""), false, true);
    match result {
        None => assert!(false),
        Some(v) => assert!(matches!(v, IOError::InvalidFilePath(_))),
    }
}

#[test]
fn test_unknown_command_error_a() {
    const PROG: &str = "foo";
    let result = construct(String::from(PROG));
    match result {
        Ok(_) => assert!(false),
        Err(v) => assert!(matches!(v, ConstructorError::UnknownCommandError(_, _, 1))),
    }
}

#[test]
fn test_unknown_command_error_b() {
    const PROG: &str = "feature_def\nfoo";
    let result = construct(String::from(PROG));
    match result {
        Ok(_) => assert!(false),
        Err(v) => assert!(matches!(v, ConstructorError::UnknownCommandError(_, _, 2))),
    }
}

#[test]
fn test_unknown_command_error_c() {
    const PROG: &str = "symbols\nfoo";
    let result = construct(String::from(PROG));
    match result {
        Ok(_) => assert!(false),
        Err(v) => assert!(matches!(v, ConstructorError::UnknownCommandError(_, _, 2))),
    }
}

#[test]
fn test_unknown_command_error_d() {
    const PROG: &str = "diacritics\nfoo";
    let result = construct(String::from(PROG));
    match result {
        Ok(_) => assert!(false),
        Err(v) => assert!(matches!(v, ConstructorError::UnknownCommandError(_, _, 2))),
    }
}

#[test]
fn test_unknown_command_error_e() {
    const PROG: &str = "rules\nfoo";
    let result = construct(String::from(PROG));
    match result {
        Ok(_) => assert!(false),
        Err(v) => assert!(matches!(v, ConstructorError::UnknownCommandError(_, _, 2))),
    }
}

#[test]
fn test_hanging_section_error_a() {
    const PROG: &str = "feature_def";
    let result = construct(String::from(PROG));
    match result {
        Ok(_) => assert!(false),
        Err(v) => assert!(matches!(v, ConstructorError::HangingSection(_, _, 1))),
    }
}

#[test]
fn test_hanging_section_error_b() {
    const PROG: &str = "symbols";
    let result = construct(String::from(PROG));
    match result {
        Ok(_) => assert!(false),
        Err(v) => assert!(matches!(v, ConstructorError::HangingSection(_, _, 1))),
    }
}

#[test]
fn test_hanging_section_error_c() {
    const PROG: &str = "diacritics";
    let result = construct(String::from(PROG));
    match result {
        Ok(_) => assert!(false),
        Err(v) => assert!(matches!(v, ConstructorError::HangingSection(_, _, 1))),
    }
}

#[test]
fn test_hanging_section_error_d() {
    const PROG: &str = "rules";
    let result = construct(String::from(PROG));
    match result {
        Ok(_) => assert!(false),
        Err(v) => assert!(matches!(v, ConstructorError::HangingSection(_, _, 1))),
    }
}

#[test]
fn test_hanging_section_error_e() {
    const PROG: &str = "rules\nrule";
    let result = construct(String::from(PROG));
    match result {
        Ok(_) => assert!(false),
        Err(v) => assert!(matches!(v, ConstructorError::HangingSection(_, _, 2))),
    }
}

#[test]
fn test_malformed_feature_def_error_a() {
    const PROG: &str = "feature_def\nswitch a(b, c)";
    let result = construct(String::from(PROG));
    match result {
        Ok(_) => assert!(false),
        Err(v) => assert!(matches!(v, ConstructorError::MalformedDefinition(_, _, 2))),
    }
}

#[test]
fn test_malformed_feature_def_error_b() {
    const PROG: &str = "feature_def\nfeature a(b, c)";
    let result = construct(String::from(PROG));
    match result {
        Ok(_) => assert!(false),
        Err(v) => assert!(matches!(v, ConstructorError::MalformedDefinition(_, _, 2))),
    }
}

#[test]
fn test_missing_node_error() {
    const PROG: &str = "feature_def\nfeature a(b, c) d";
    let result = construct(String::from(PROG));
    match result {
        Ok(_) => assert!(false),
        Err(v) => assert!(matches!(v, ConstructorError::MissingNode(_, _, 2))),
    }
}

fn is_anagram(a: String, b: String) -> bool {
    let mut avec: Vec<char> = a.chars().collect();
    avec.sort();
    let mut bvec: Vec<char> = b.chars().collect();
    bvec.sort();
    return avec == bvec;
}

fn create_diacritic_test_program() -> Program {
    construct(load_from_file(&String::from("test-data/diacritics-test.lsc"), false).expect("Error reading file")).unwrap()
}

fn create_int_test_1() -> Program {
    let defs = load_from_file(&String::from("test-data/full-ipa.lsc"), false).expect("Error reading file");
    let rules = load_from_file(&String::from("test-data/int-test-1.lsc"), false).expect("Error reading file");
    construct(format!("{0}\n{1}", defs, rules)).unwrap()
}

fn random_letter() -> super::data::Letter {
    let letter = random_u64();
    super::data::Letter{ value: letter }
}

fn random_u64() -> u64 {
    rand::thread_rng().gen()
}