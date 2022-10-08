use super::data::*;
use super::rules::*;

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

fn random_letter() -> super::data::Letter {
    let letter = random_u64();
    super::data::Letter{ value: letter }
}

fn random_u64() -> u64 {
    rand::thread_rng().gen()
}

// fn random_character() -> char {
//     rand::thread_rng().sample_iter(&Alphanumeric).take(1).map(char::from).next().unwrap()
// }
