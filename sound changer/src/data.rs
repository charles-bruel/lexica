use std::{collections::HashMap};
use super::priority_queue::PriorityQueue;

pub struct Program {
    pub features: Vec<Feature>,
    pub diacritics: Vec<Diacritic>,
    pub rules: Vec<Rule>,
    pub names_to_idx: HashMap<String, u32>,
    pub idx_to_features: HashMap<u32, Feature>,
    pub features_to_idx: HashMap<String, (u32, usize)>,
    pub symbol_to_letter: HashMap<String, (Letter, u64)>,
    pub letter_to_symbol: HashMap<Letter, String>,
}

#[derive(Clone)]
pub enum Feature {
    SwitchType(SwitchType),
    FeatureDef(FeatureDef),
}

#[derive(Clone)]
pub struct SwitchType {
    pub start_byte: u8,
    pub tot_length: u8,
    pub self_length: u8,
    pub features: Vec<Vec<Feature>>,
    pub name: String,
    pub option_names: Vec<String>,
    pub id: u32,
    pub validation_mask: u64,
    pub validation_key: u64,
}

#[derive(Clone)]
pub struct FeatureDef {
    pub start_byte: u8,
    pub length: u8,
    pub name: String,
    pub option_names: Vec<String>,
    pub is_bool: bool,
    pub id: u32,
    pub validation_mask: u64,
    pub validation_key: u64,
}

impl Feature {
    pub fn start_byte(&self) -> u8 {
        match self {
            Feature::SwitchType(data) => return data.start_byte,
            Feature::FeatureDef(data) => return data.start_byte,
        }
    }
    pub fn length(&self) -> u8 {
        match self {
            Feature::SwitchType(data) => return data.self_length,
            Feature::FeatureDef(data) => return data.length,
        }
    }
    pub fn tot_length(&self) -> u8 {
        match self {
            Feature::SwitchType(data) => return data.tot_length,
            Feature::FeatureDef(data) => return data.length,
        }
    }
    pub fn name(&self) -> String {
        match self {
            Feature::SwitchType(data) => return data.name.clone(),
            Feature::FeatureDef(data) => return data.name.clone(),
        }
    }
    pub fn id(&self) -> u32 {
        match self {
            Feature::SwitchType(data) => return data.id,
            Feature::FeatureDef(data) => return data.id,
        }
    }
    pub fn validation_key(&self) -> u64 {
        match self {
            Feature::SwitchType(data) => return data.validation_key,
            Feature::FeatureDef(data) => return data.validation_key,
        }
    }
    pub fn validation_mask(&self) -> u64 {
        match self {
            Feature::SwitchType(data) => return data.validation_mask,
            Feature::FeatureDef(data) => return data.validation_mask,
        }
    }
    pub fn clone_light(&self) -> Feature {
        match self {
            Feature::SwitchType(data) => {
                let mut temp = data.clone();
                temp.features = Vec::new();
                return Feature::SwitchType(temp);
            }
            Feature::FeatureDef(_) => self.clone(),
        }
    }
    pub fn validate(&self, letter: &Letter) -> bool {
        let validation = match self {
            Feature::SwitchType(data) => {
                (data.validation_key, data.validation_mask)
            }
            Feature::FeatureDef(data) => {
                (data.validation_key, data.validation_mask)
            }
        };
        let value = letter.value;
        return value & validation.1 == validation.0;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Letter {
    pub value: u64
}

impl Letter {
    pub fn get_symbol(&self, program: &Program) -> String{
        let temp = program.letter_to_symbol.get(self);
        match temp {
            Some(result) => result.to_string(),
            None => {
                let mut queue: PriorityQueue<(u64, u64, &str), i8> = PriorityQueue::new();
                queue.push((self.value, self.value, ""), 0);
                let mut completed_nodes: HashMap<u64, (u64, &str)> = HashMap::new();
                let mut depth: u16 = 0;
                while depth < 128 {
                    if queue.is_empty() {
                        panic!("Could not find matching symbol for {:#066b}", self.value);
                    }
                    let ((value, prev_node, current_symbol), priority) = queue.pop().unwrap();

                    for diacritic in &program.diacritics {
                        if diacritic.mask & value == diacritic.mod_key {
                            let new_value = (value & !diacritic.mask ) | diacritic.key;
                            queue.push((new_value, value, &diacritic.diacritic), priority - 1);
                        }
                    }

                    if prev_node != value {
                        completed_nodes.insert(value, (prev_node, current_symbol));
                    }

                    let letter = Letter { value: value };
                    if program.letter_to_symbol.contains_key(&letter){
                        let mut result = letter.get_symbol(program);
                        let mut current_node = value;
                        let mut depth2: u16 = 0;
                        while depth2 < 128 {
                            let (new_node, symbol) = match completed_nodes.get(&current_node) {
                                Some(val) => val,
                                None => return result,
                            };
                            current_node = *new_node;
                            result.push_str(symbol);
                            depth2 += 1;
                        }
                        panic!("Could not find matching symbol for {:#066b}", self.value);
                    }

                    depth += 1
                }
                
                panic!("Could not find matching symbol for {:#066b}", self.value);
            },
        }
    }
    pub fn extract_property(&self, prop: Feature) -> u64 {
        let mut result = self.value;
        result <<= prop.start_byte();
        let mask: u64 = !(0xFFFFFFFFFFFFFFFF >> prop.length());
        result &= mask;
        result >>= 64 - prop.length();
        return result;
    }
}

pub trait Predicate {
    fn validate(&self, word: &Vec<Letter>, position: usize) -> bool;
}

pub trait Result {
    fn transform(&self, input: &Letter) -> Option<Letter>;
}

pub struct Transformation {
    pub predicate: Vec<Box<dyn Predicate>>,
    pub result: Vec<Box<dyn Result>>,
    pub predicate_captures: Vec<(usize, u64)>,
    pub result_captures: Vec<usize>,
}

pub struct RuleByte {
    pub transformations: Vec<Transformation>,
    pub enviorment: Enviorment,
    pub num_captures: usize,
}

pub struct Rule {
    pub bytes: Vec<RuleByte>,
    pub flags: u16,
    pub name: String,
}

pub struct EnviormentPredicate {
    pub predicate: Box<dyn Predicate>,
    pub min_quant: u8,
    pub max_quant: u8,
}

pub struct Enviorment {
    pub ante: Vec<EnviormentPredicate>,
    pub post: Vec<EnviormentPredicate>,
    pub ante_word_boundary: bool,
    pub post_word_boundary: bool,
}

pub struct Diacritic {
    pub diacritic: String,
    pub mask: u64,
    pub key: u64,
    pub mod_key: u64,
}

#[derive(PartialEq)]
pub enum Ordering {
    Forward, Reverse
}

static mut FEATURE_ID_TRACKER: u32 = 0;
//Technically unsafe but the program generation will not occur in threads
fn get_id() -> u32 {
    unsafe {
        FEATURE_ID_TRACKER += 1;
        return FEATURE_ID_TRACKER;
    }
}

pub fn create_empty_program() -> Program {
    Program { 
        features: Vec::new(),
        diacritics: Vec::new(),
        rules: Vec::new(),
        names_to_idx: HashMap::new(),
        idx_to_features: HashMap::new(),
        features_to_idx: HashMap::new(),
        letter_to_symbol: HashMap::new(),
        symbol_to_letter: HashMap::new(),
    }
}

pub fn create_feature_def_bool(name: String, negative_option: String, positive_option: String) -> FeatureDef {
    let id = get_id();
    let temp = FeatureDef {
        start_byte: 0,
        length: 1,
        name: name,
        option_names: vec![negative_option, positive_option],
        is_bool: true,
        id: id,
        validation_key: 0,
        validation_mask: 0,
    };

    return temp;
}

pub fn create_feature_def(name: String, option_names: Vec<String>) -> FeatureDef {
    let len = f64::from(option_names.len() as u32).log2().ceil() as u8;
    let id = get_id();
    let temp = FeatureDef {
        start_byte: 0,
        length: len,
        name: name,
        option_names: option_names,
        is_bool: false,
        id: id,
        validation_key: 0,
        validation_mask: 0,
    };

    return temp;
}

pub fn create_switch_type(name: String, option_names: Vec<String>, features: Vec<Vec<Feature>>) -> SwitchType {
    let len = f64::from(option_names.len() as u32).log2().ceil() as u8;
    let id = get_id();
    let temp = SwitchType {
        start_byte: 0,
        self_length: len,
        tot_length: 0,
        features: features,
        name: name,
        option_names: option_names,
        id: id,
        validation_key: 0,
        validation_mask: 0,
    };

    return temp;
}

pub fn create_diacritic(diacritic: String, mask: u64, key: u64, mod_key: u64,) -> Diacritic {
    return Diacritic {
        diacritic: diacritic,
        mask: mask,
        key: key,
        mod_key: mod_key,
    };
}

impl Program {
    pub fn print_structure(&self){
        print_structure_recurse(&self.features, 0);
    }
}

fn print_structure_recurse(features: &Vec<Feature>, level: u8){
    for f in features {
        let whitespace: String = String::from_utf8(vec![b'\t'; usize::from(level)]).ok().unwrap();
        match f { 
            Feature::SwitchType(data) => {
                print!("{}", whitespace);
                print!("Name: {0}, Start byte: {1}, Self Length: {2}, Total Length {3}\n", data.name, data.start_byte, data.self_length, data.tot_length);
                let mut i: usize = 0;
                while i < data.option_names.len() {
                    if data.features[i].len() != 0 {
                        print!("{}", whitespace);
                        print!("Option: {}:\n", data.option_names[i]);
                        print_structure_recurse(&data.features[i], level + 1);
                    }
                    i += 1;
                }
            }
            Feature::FeatureDef(data) => {
                print!("{}", whitespace);
                print!("Name: {0}, Start byte: {1}, Length: {2}\n", data.name, data.start_byte, data.length);
            } 
        }
    }
}

pub fn create_rule_byte(predicate: (Vec<Box<dyn Predicate>>, Vec<(usize, u64)>), result: (Vec<Box<dyn Result>>, Vec<usize>), enviorment: Enviorment) -> RuleByte {
    let mut num_captures: usize = 0;

    for x in &predicate.1 {
        if num_captures < x.0 {
            num_captures = x.0;
        }
    }

    for x in &result.1 {
        if *x > num_captures {
            panic!("More output captures than input captures");
        }
    }

    RuleByte {
        transformations: vec![
            Transformation {
                predicate: predicate.0,
                result: result.0,
                predicate_captures: predicate.1,
                result_captures: result.1,
            }
        ],
        enviorment: enviorment,
        num_captures: num_captures + 1,
    }
}

pub fn create_multi_rule_byte(predicate: Vec<(Vec<Box<dyn Predicate>>, Vec<(usize, u64)>)>, result: Vec<(Vec<Box<dyn Result>>, Vec<usize>)>, enviorment: Enviorment) -> RuleByte {
    assert_eq!(predicate.len(), result.len());
    let mut transformations: Vec<Transformation> = Vec::new();

    let mut i: usize = 0;
    while i < predicate.len() {
        transformations.push(
            Transformation {
                predicate: Vec::new(),
                result: Vec::new(),
                predicate_captures: Vec::new(),
                result_captures: Vec::new(),
            }
        );
        i += 1;
    }

    let mut num_captures: usize = 0;

    i = 0;
    for p in predicate {
        transformations[i].predicate = p.0;

        for x in &p.1 {
            if num_captures < x.0 {
                num_captures = x.0;
            }
        }
        transformations[i].predicate_captures = p.1;
        
        i += 1;
    }

    i = 0;
    for r in result {
        transformations[i].result = r.0;

        for x in &r.1 {
            if *x > num_captures {
                panic!("More output captures than input captures");
            }
        }
        transformations[i].result_captures = r.1;

        i += 1;
    }

    
    RuleByte {
        transformations: transformations,
        enviorment: enviorment,
        num_captures: num_captures + 1,
    }
}

pub fn create_rule(name: String, bytes:Vec<RuleByte>, flags: u16) -> Rule {
    Rule { 
        bytes: bytes,
        flags: flags,
        name: name,
    }
}

pub fn create_empty_enviorment() -> Enviorment {
    Enviorment {
        ante: Vec::new(),
        post: Vec::new(),
        ante_word_boundary: false,
        post_word_boundary: false,
    }
}

pub fn create_enviorment(ante: Vec<EnviormentPredicate>, post: Vec<EnviormentPredicate>, ante_word_boundary: bool, post_word_boundary: bool) -> Enviorment {
    Enviorment {
        ante: ante,
        post: post,
        ante_word_boundary: ante_word_boundary,
        post_word_boundary: post_word_boundary,
    }
}

pub fn create_enviorment_predicate_single(predicate: Box<dyn Predicate>) -> EnviormentPredicate {
    EnviormentPredicate {
        predicate: predicate,
        min_quant: 1,
        max_quant: 1,
    }
}

pub fn create_enviorment_predicate(predicate: Box<dyn Predicate>, min: u8, max: u8) -> EnviormentPredicate {
    EnviormentPredicate {
        predicate: predicate,
        min_quant: min,
        max_quant: max,
    }
}