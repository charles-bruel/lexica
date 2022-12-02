use data::*;

pub struct MultiPredicate {
    pub predicate: Vec<Box<dyn Predicate>>,
    pub req_all: bool,
}

pub struct SimplePredicate {
    pub mask: u64,
    pub key: u64,
}

pub struct PositiveNegativePredicate {
    pub positive_mask: u64,
    pub positive_key: u64,
    pub negative_masks: Vec<u64>,
    pub negative_keys: Vec<u64>,
}

pub struct SimpleResult {
    pub letter: Letter,
}

pub struct SimpleApplicationResult {
    pub mask: u64,
    pub value: u64,
}

pub struct DeleteResult {}

impl Predicate for MultiPredicate {
    fn validate(&self, word: &Word, position: usize) -> bool {
        let mut flag: bool = self.req_all;
        for predicate in &self.predicate {
            if self.req_all != predicate.validate(word, position) { flag = !self.req_all; }
        }
        return flag;
    }
}

impl Predicate for SimplePredicate {
    fn validate(&self, word: &Word, position: usize) -> bool {
        let letter = word[position];
        return (letter.value & self.mask) == self.key;
    }
}

impl Predicate for PositiveNegativePredicate {
    fn validate(&self, word: &Word, position: usize) -> bool {
        let letter = word[position];

        if (letter.value & self.positive_mask) != self.positive_key {
            return false;
        }

        let mut i: usize = 0;
        while i < self.negative_masks.len() {
            if (letter.value & self.negative_masks[i]) == self.negative_keys[i] {
                return false;
            }
            i += 1;
        }
        return true;
    }
}

impl Result for SimpleResult {
    fn transform(&self, _input: &Letter) -> Option<Letter> {
        return Some(self.letter);
    }
}

impl Result for SimpleApplicationResult {
    fn transform(&self, input: &Letter) -> Option<Letter> {
        let value = (input.value & !self.mask) | self.value;
        return Some(Letter { value: value });
    }
}

impl Result for DeleteResult {
    fn transform(&self, _input: &Letter) -> Option<Letter> { None }
}

pub fn create_multi_predicate(predicates: Vec<Box<dyn Predicate>>, req_all: bool) -> MultiPredicate {
    MultiPredicate {
        predicate: predicates,
        req_all: req_all,
    }
}


pub fn create_simple_predicate(key: u64, mask: u64) -> SimplePredicate {
    SimplePredicate {
        key: key,
        mask: mask,
    }
}

pub fn create_positive_negative_predicate(positive_mask: u64, positive_key: u64, negative_masks: Vec<u64>, negative_keys: Vec<u64>) -> PositiveNegativePredicate {
    if negative_masks.len() != negative_keys.len() {
        panic!("Mismatched number of masks and keys");
    }
    PositiveNegativePredicate {
        positive_mask: positive_mask,
        positive_key: positive_key,
        negative_masks: negative_masks,
        negative_keys: negative_keys,
    }
}

pub fn create_simple_result(letter: Letter) -> SimpleResult {
    SimpleResult {
        letter: letter,
    }
}

pub fn create_simple_application_result(mask: u64, value: u64) -> SimpleApplicationResult {
    SimpleApplicationResult {
        mask: mask,
        value: value,
    }
}

pub fn create_delete_result() -> DeleteResult {
    DeleteResult {}
}