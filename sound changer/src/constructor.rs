use std::{collections::*, vec};

use {data::*, rules::*, applicator::*};
use super::fancy_regex::Regex;

enum State {
    None,
    Features,
    Symbols,
    Diacritics,
    Rules,
    RuleAccum,
}

pub fn construct(input: String) -> std::result::Result<Program, ConstructorError> {
    use std::time::Instant;
    let now = Instant::now();


    let mut current_state = State::None;
    let mut program = create_empty_program();
    let lines: Vec<&str> = input.split("\n").collect();

    let mut rule_accum: Vec<&str> = Vec::new();

    let mut line_number: u16 = 0;

    for f in lines {
        line_number += 1;

        let line_og = f.clone();
        let mut line = line_og.trim();

        if line.contains("#") {
            let temp: Vec<&str> = line.split("#").collect();
            line = temp[0];
        }
        
        let regex = Regex::new(r" (?![^(]*\))(?![^\[]*\])").unwrap();
        let mut temp = regex.replace_all(line, String::from_utf8(vec![0]).unwrap());
        let words: Vec<&str> = temp.to_mut().split('\u{0000}').collect();

        match current_state {
            State::None => {
                if words[0] == "feature_def" {
                    current_state = State::Features;
                } else if words[0] == "symbols" {
                    current_state = State::Symbols;
                } else if words[0] == "rules" {
                    current_state = State::Rules;
                } else if words[0] == "diacritics" {
                    current_state = State::Diacritics;
                } else if words[0] != "" {
                    return Err(ConstructorError::UnknownCommandError(format!("Unknown command \"{}\"", words[0]), String::from(line_og), line_number))
                }
            },
            State::Features => {
                if words[0] == "switch" {
                    handle_err(construct_switch_line(&mut program, &words), String::from(line_og), line_number)?;
                } else if words[0] == "feature" {
                    handle_err(construct_feature_def(&mut program, &words), String::from(line_og), line_number)?;
                } else if words[0] == "end" {
                    end_feature_def(&mut program);
                    current_state = State::None;
                } else if words[0] != "" {
                    return Err(ConstructorError::UnknownCommandError(format!("Unknown command \"{}\"", words[0]), String::from(line_og), line_number))
                }
            },
            State::Symbols => {
                if words[0] == "symbol" {
                    construct_symbol(&mut program, &words);
                } else if words[0] == "end" {
                    current_state = State::None;
                } else if words[0] != "" {
                    return Err(ConstructorError::UnknownCommandError(format!("Unknown command \"{}\"", words[0]), String::from(line_og), line_number))
                }
            },
            State::Rules => {
                if words[0] == "rule" {
                    rule_accum.push(line);
                    current_state = State::RuleAccum;
                } else if words[0] == "end" {
                    end_feature_def(&mut program);
                    current_state = State::None;
                } else if words[0] != "" {
                    return Err(ConstructorError::UnknownCommandError(format!("Unknown command \"{}\"", words[0]), String::from(line_og), line_number))
                }
            },
            State::RuleAccum => {
                if words[0] == "end" {
                    construct_rule(&mut program, rule_accum);
                    rule_accum = Vec::new();
                    current_state = State::Rules;
                } else {
                    rule_accum.push(line);
                }
            },
            State::Diacritics => {
                if words[0] == "diacritic" {
                    construct_diacritic(&mut program, &words);
                } else if words[0] == "end" {
                    current_state = State::None;
                } else if words[0] != "" {
                    return Err(ConstructorError::UnknownCommandError(format!("Unknown command \"{}\"", words[0]), String::from(line_og), line_number))
                }
            },
        }
    }

    match current_state {
        State::None => {},
        State::Features => return Err(ConstructorError::HangingSection(String::from("Features section never finishes"), String::from("EOF"), line_number)),
        State::Symbols => return Err(ConstructorError::HangingSection(String::from("Features section never finishes"), String::from("EOF"), line_number)),
        State::Diacritics => return Err(ConstructorError::HangingSection(String::from("Features section never finishes"), String::from("EOF"), line_number)),
        State::Rules => return Err(ConstructorError::HangingSection(String::from("Features section never finishes"), String::from("EOF"), line_number)),
        State::RuleAccum => return Err(ConstructorError::HangingSection(String::from("Features section never finishes"), String::from("EOF"), line_number)),
    }

    let elapsed = now.elapsed();
    print!("Done loading and constructing program in {:.2?}\n", elapsed);

    return Ok(program);
}

fn handle_err(result: std::result::Result<(), ConstructorError>, line: String, line_number: u16) -> std::result::Result<(), ConstructorError> {
    match result {
        Ok(_) => Ok(()),
        Err(v) => {
            Err(match v {
                ConstructorError::UnknownCommandError(m, _, _) => ConstructorError::UnknownCommandError(m, line, line_number),
                ConstructorError::HangingSection(m, _, _) => ConstructorError::HangingSection(m, line, line_number),
                ConstructorError::MalformedDefinition(m, _, _) => ConstructorError::MalformedDefinition(m, line, line_number),
                ConstructorError::MissingNode(m, _, _) => ConstructorError::MissingNode(m, line, line_number),
            })
        }
    }
}

pub fn construct_words(program: &Program, input: String) -> std::result::Result<Vec<Vec<Letter>>, ApplicationError> {
    let lines: Vec<&str> = input.split("\n").collect();
    let mut result: Vec<Vec<Letter>> = Vec::new();
    for l in lines {
        result.push(from_string(&program, &String::from(l.trim()))?);
    }
    return Ok(result);
}

fn construct_diacritic(program: &mut Program, line: &Vec<&str>) {
    if line.len() != 5 || line[3] != "=>" {
        panic!("Malformed diacritic definition");
    }
    let mut symbol = String::from(line[1]);
    symbol.remove_matches("◌");
    let (mask, key) = parse_features_simple(program, line[2]);
    let (mod_mask, mod_key) = parse_features_simple(program, line[4]);

    if mask != mod_mask {
        panic!("Features don't have the same mask for diacritic");
    }

    let diacritic = create_diacritic(symbol, mask, key, mod_key);
    program.diacritics.push(diacritic);
}

fn construct_rule(program: &mut Program, line: Vec<&str>){
    if line.len() < 2 {
        panic!("Malformed rule, header: \"{}\"", line[0]);
    }
    let (name, flags) = construct_rule_header(line[0]);

    let mut i: usize = 1;
    let mut rule_bytes: Vec<RuleByte> = Vec::new();
    while i < line.len() {
        rule_bytes.push(construct_rule_byte(program, line[i]));
        i += 1;
    }

    let temp = create_rule(name, rule_bytes, flags);
    program.rules.push(temp);
}

fn construct_rule_byte(program: &Program, data: &str) -> RuleByte {
    let split1: Vec<&str> = data.split("=>").collect();
    if split1.len() != 2 {
        panic!("Malformed rule byte on line: \"{}\"", data);
    }

    let split2: Vec<&str> = split1[1].split("/").collect();
    if split2.len() > 2 {
        panic!("Malformed rule byte on line: \"{}\"", data);
    }

    let predicate = split1[0].trim();
    let (result, enviorment) = match split2.len() {
        1 => { (split2[0].trim(), "") }
        2 => { (split2[0].trim(), split2[1].trim()) }
        _ => { panic!("Malformed rule byte on line: \"{}\"", data); }
    };

    let regex = Regex::new(r" (?![^(]*\))(?![^\[]*\])(?![^\{]*\})").unwrap();
    let mut temp = regex.replace_all(predicate, String::from_utf8(vec![0]).unwrap());
    let predicate_split: Vec<&str> = temp.to_mut().split('\u{0000}').collect();
    let mut temp = regex.replace_all(result, String::from_utf8(vec![0]).unwrap());
    let result_split: Vec<&str> = temp.to_mut().split('\u{0000}').collect();

    if predicate_split.len() != result_split.len() {
        panic!("Predicate and result have a different number of elements on rule \"{}\"", data);
    }

    if predicate_split.len() > 1 {
        let mut i: usize = 0;
        let mut predicates: Vec<(Vec<Box<dyn Predicate>>, Vec<(usize, u64)>)> = Vec::new();
        let mut results: Vec<(Vec<Box<dyn Result>>, Vec<usize>)> = Vec::new();
        while i < predicate_split.len() {
            predicates.push(construct_predicate(program, predicate_split[i]));
            results.push(construct_result(program, result_split[i]));
            i += 1;
        }
        return create_multi_rule_byte(predicates, results, construct_enviorment(program, enviorment));
    } else {
        return create_rule_byte(construct_predicate(program, predicate), construct_result(program, result), construct_enviorment(program, enviorment));
    }
}

fn construct_predicate(program: &Program, predicate: &str) -> (Vec<Box<dyn Predicate>>, Vec<(usize, u64)>) {
    let mut input = predicate.trim();

    let mut captures: Vec<(usize, u64)> = Vec::new();
    if input.contains("$") {
        let temp: Vec<&str> = input.split("$").collect();
        input = temp[0];
        let mut i: usize = 1;
        while i < temp.len() {
            let val = construct_capture(program, temp[i].trim());
            captures.push(val);
            i += 1;
        }
    }

    if input == "*" { 
        return (Vec::new(), Vec::new());
    }
    if input.starts_with("{") && input.ends_with("}") {
        input = input.trim_end_matches("}").trim_start_matches("{");
        let results = construct_predicates(program, input);
        return (results, captures);
    }
    if input.contains("{") || input.contains("}") {
        panic!("Malformed predicate: \"{}\"", input);
    }
    if input.starts_with("(") && input.ends_with(")") {
        input = input.trim_end_matches(")").trim_start_matches("(");
        let results = construct_predicates(program, input);
        let multi_predicate = create_multi_predicate(results, false);
        return (vec!(Box::new(multi_predicate)), captures);
    }
    if input.contains("(") || input.contains(")") {
        panic!("Malformed predicate: \"{}\"", input);
    }

    return (vec!(construct_simple_predicate(program, input)), captures);
}

fn construct_predicates(program: &Program, input: &str) -> Vec<Box<dyn Predicate>> {
    let regex = Regex::new(r" (?![^(]*\))(?![^\[]*\])(?![^\{]*\})").unwrap();
    let mut temp = regex.replace_all(input, String::from_utf8(vec![0]).unwrap());
    let input_split: Vec<&str> = temp.to_mut().split('\u{0000}').collect();

    let mut results: Vec<Box<dyn Predicate>> = Vec::new();
    for x in input_split {
        let (mut pred, _) = construct_predicate(program, x);
        results.append(&mut pred);
    }

    return results;
}

fn construct_capture(program: &Program, capture: &str) -> (usize, u64) {
    if capture.contains("(") {
        if !capture.ends_with(")") {
            panic!("Malformed capture \"{}\"", capture);
        }

        let split: Vec<&str> = capture.split("(").collect();

        let id = program.names_to_idx.get(split[1].trim_end_matches(")"));

        let feature = match id {
            Some(val) => {
                program.idx_to_features.get(val).unwrap()
            },
            None => panic!("Could find feature in capture \"{}\"", capture),
        };

        let offset = 64 - feature.start_byte() - feature.length();
        let mask = ((2 << feature.length() - 1) - 1) << offset;

        let temp = split[0].parse::<usize>();
        match temp {
            Ok(val) => return (val, mask),
            Err(_) => panic!("Could not read capture id \"{}\"", capture),
        }
    } else {
        let temp = capture.parse::<usize>();
        match temp {
            Ok(val) => return (val, 0xFFFFFFFFFFFFFFFF),
            Err(_) => panic!("Could not read capture id \"{}\"", capture),
        }
    }
}

pub(crate) fn construct_simple_predicate(program: &Program, predicate: &str) -> Box<dyn Predicate> {
    if predicate.starts_with("[") && predicate.ends_with("]") {
        if predicate.contains("!") {
            let (mask, key, masks, keys) = parse_features_negative(program, predicate);
            let predicate = create_positive_negative_predicate(mask, key, masks, keys);
            return Box::new(predicate);  
        } else {
            let (mask, key) = parse_features(program, predicate);
            let predicate = create_simple_predicate(key, mask);
            return Box::new(predicate);    
        }
    }
    if predicate.contains("[") || predicate.contains("]") {
        panic!("Malformed predicate: \"{}\"", predicate);
    }

    if program.symbol_to_letter.contains_key(predicate) {
        let (letter, mask) = program.symbol_to_letter.get(predicate).unwrap();
        let predicate = create_simple_predicate(letter.value, *mask);
        return Box::new(predicate);
    }
    panic!("Could not find symbol \"{}\"", predicate);
}

fn construct_result(program: &Program, result: &str) -> (Vec<Box<dyn Result>>, Vec<usize>) {
    let mut input = result.trim();

    let mut captures: Vec<usize> = Vec::new();
    if input.contains("$") {
        let temp: Vec<&str> = input.split("$").collect();
        input = temp[0];
        let mut i: usize = 1;
        while i < temp.len() {
            let (val, _) = construct_capture(program, temp[i].trim());
            captures.push(val);
            i += 1;
        }
    }

    if input.starts_with("{") && input.ends_with("}") {
        input = input.trim_end_matches("}").trim_start_matches("{");
        let results = construct_results(program, input);
        return (results, captures);
    }
    if input.contains("{") || input.contains("}") {
        panic!("Malformed result: \"{}\"", input);
    }

    return (vec!(construct_single_result(program, input)), captures);
}

fn construct_results(program: &Program, input: &str) -> Vec<Box<dyn Result>> {
    let regex = Regex::new(r" (?![^(]*\))(?![^\[]*\])(?![^\{]*\})").unwrap();
    let mut temp = regex.replace_all(input, String::from_utf8(vec![0]).unwrap());
    let input_split: Vec<&str> = temp.to_mut().split('\u{0000}').collect();

    let mut results: Vec<Box<dyn Result>> = Vec::new();
    for x in input_split {
        let (mut pred, _) = construct_result(program, x);
        results.append(&mut pred);
    }

    return results;
}

fn construct_single_result(program: &Program, result: &str) -> Box<dyn Result> {
    if result.starts_with(">[") || result.starts_with("[") && result.ends_with("]") {
        if result.starts_with(">") {
            let temp = result.trim_start_matches(">");
            let (_, value) = parse_features(program, temp);
            let result = create_simple_result(Letter { value: value });
            return Box::new(result);
        } else {
            let (mask, value) = parse_features_simple(program, result);
            let result = create_simple_application_result(mask, value);
            return Box::new(result);
        }
    }
    if result.contains(">") || result.contains("[") || result.contains("]") {
        panic!("Malformed result: \"{}\"", result);
    }

    if result == "*" {
        return Box::new(create_delete_result());
    }

    if program.symbol_to_letter.contains_key(result) {
        let (letter, _) = program.symbol_to_letter.get(result).unwrap();
        let result = create_simple_result(letter.clone());
        return Box::new(result);
    }
    panic!("Could not find symbol \"{}\"", result);
}

fn parse_features_simple(program: &Program, features: &str) -> (u64, u64) {
    let mut feature = features;
    if feature.starts_with("[") {
        feature = feature.trim_start_matches("[");
    }
    if feature.ends_with("]") {
        feature = feature.trim_end_matches("]");
    }
    
    let params: Vec<&str> = feature.split_whitespace().collect();

    let mut mask: u64 = 0;
    let mut key: u64 = 0;

    let mut i: usize = 0;

    while i < params.len() {
        let p = params[i];
        let temp = program.features_to_idx.get(p);
        let (idx, index) = match temp {
            Some(data) => data,
            None => panic!("Unknown feature \"{}\"", p),
        };
        
        let feature = program.idx_to_features.get(idx).unwrap();

        let offset = 64 - feature.start_byte() - feature.length();
        mask |= ((2 << feature.length() - 1) - 1) << offset;
        key |= (*index as u64) << offset;

        i += 1;
    }

    return (mask, key);
}

fn construct_enviorment(program: &Program, enviorment: &str) -> Enviorment {
    if enviorment == "" {
        return create_empty_enviorment();
    }
    if !enviorment.contains("_") {
        panic!("Malformed enviorment \"{}\"", enviorment);
    }

    let enviorment_wings: Vec<&str> = enviorment.split("_").collect();
    if enviorment_wings.len() != 2 {
        panic!("Malformed enviorment \"{}\"", enviorment);
    }

    let (ante_wing, ante_boundary) = construct_enviorment_wing(program, enviorment_wings[0], Ordering::Reverse);
    let (post_wing, post_boundary) = construct_enviorment_wing(program, enviorment_wings[1], Ordering::Forward);

    return create_enviorment(ante_wing, post_wing, ante_boundary, post_boundary);
}

fn construct_enviorment_wing(program: &Program, enviorment: &str, direction: Ordering) -> (Vec<EnviormentPredicate>, bool) {
    let working_value = enviorment.trim();

    let regex = Regex::new(r" (?![^(]*\))(?![^\[]*\])").unwrap();
    let mut temp = regex.replace_all(working_value, String::from_utf8(vec![0]).unwrap());
    let mut enviorment_components: Vec<&str> = temp.to_mut().split('\u{0000}').collect();
    
    if direction == Ordering::Reverse {
        enviorment_components.reverse();
    }

    let mut result: Vec<EnviormentPredicate> = Vec::new();

    let mut flag = false;

    for c in enviorment_components {
        if c == "" {
            continue;
        }
        if c == "$" {
            flag = true;
        } else {
            if flag {
                panic!("Word boundary condition in middle of enviorment \"{}\"", enviorment);
            }
            result.push(construct_enviorment_predicate(program, c));
        }
    }

    return (result, flag);
}

fn construct_enviorment_predicate(program: &Program, predicate: &str) -> EnviormentPredicate {
    if predicate.contains("<") {
        let predicate_split: Vec<&str> = predicate.split("<").collect();
        if predicate_split.len() != 2 {
            panic!("Malformed enviorment predicate \"{}\"", predicate);
        }
        let predicate_instance = construct_simple_predicate(program, predicate_split[0]);

        let quantity_spec = predicate_split[1].trim_end_matches(">");
        let quantities: Vec<&str> = quantity_spec.split(":").collect();
        if quantities.len() != 2 {
            panic!("Malformed quantity specifier on \"{}\"", predicate);
        }

        let quant_min = quantities[0].parse::<u8>();
        let quant_max = quantities[1].parse::<u8>();

        let quant_min_value = match quant_min {
            Ok(val) => val,
            Err(_) => panic!("Malformed quantity specifier on \"{}\"", predicate),
        };
        let quant_max_value = match quant_max {
            Ok(val) => val,
            Err(_) => panic!("Malformed quantity specifier on \"{}\"", predicate),
        };

        return create_enviorment_predicate(predicate_instance, quant_min_value, quant_max_value);
    }

    let predicate_features = predicate.trim_end_matches(&['+', '*', '+']);
    let predicate_instance = construct_simple_predicate(program, predicate_features);
    if predicate.ends_with("?") {
        return create_enviorment_predicate(predicate_instance, 0, 1);
    }
    if predicate.ends_with("*") {
        return create_enviorment_predicate(predicate_instance, 0, 255);
    }
    if predicate.ends_with("+") {
        return create_enviorment_predicate(predicate_instance, 1, 255);
    }
    return create_enviorment_predicate_single(predicate_instance);
}

fn construct_rule_header(data: &str) -> (String, u16) {
    let words: Vec<&str> = data.split_whitespace().collect();

    if words.len() < 2 {
        panic!("Malformed rule header on line \"{}\"", data);
    }

    let name = String::from(words[1]);

    if name.contains(&['(', ')', '+', '!', '"', ',']) {
        panic!("Invald characters in rule name \"{}\"", name);
    }

    return (name, 0);//TODO add flags
}

fn construct_symbol(program: &mut Program, line: &Vec<&str>) {
    if line.len() != 3 {
        panic!("Malformed symbol definition on line \"{}\"", line.join(" "));
    }

    let symbol = line[1];
    if symbol.contains(&['(', ')', '+', '!', '"', ',']) {
        panic!("Invald characters in symbol \"{}\"", symbol);
    }

    let (mask, value) =  parse_features(&program, line[2]);
    let letter = Letter{ value: value };
    program.letter_to_symbol.insert(letter, String::from(symbol));
    program.symbol_to_letter.insert(String::from(symbol), (letter, mask));
}

fn parse_features_negative(program: &Program, features: &str) -> (u64, u64, Vec<u64>, Vec<u64>) {
    let mut feature = features;
    if feature.starts_with("[") {
        feature = feature.trim_start_matches("[");
    }
    if feature.ends_with("]") {
        feature = feature.trim_end_matches("]");
    }
    
    let params: Vec<&str> = feature.split_whitespace().collect();

    let mut mask: u64 = 0;
    let mut key: u64 = 0;
    let mut masks: Vec<u64> = Vec::new();
    let mut keys: Vec<u64> = Vec::new();

    let mut validation_key: u64 = 0;

    let mut i: usize = 0;

    while i < params.len() {
        let mut p = params[i];
        let flag = p.starts_with("!");
        p = p.trim_start_matches("!");
        let temp = program.features_to_idx.get(p);
        let (idx, index) = match temp {
            Some(data) => data,
            None => panic!("Unknown feature \"{}\"", p),
        };
        
        let feature = program.idx_to_features.get(idx).unwrap();


        validation_key |= feature.validation_key();

        let offset = 64 - feature.start_byte() - feature.length();

        if flag {
            masks.push(((2 << feature.length() - 1) - 1) << offset);
            keys.push((*index as u64) << offset);
        } else {
            mask |= ((2 << feature.length() - 1) - 1) << offset;
            key |= (*index as u64) << offset;

            mask |= feature.validation_mask();
            key |= feature.validation_key();
        }
        i += 1;
    }

    i = 0;

    while i < params.len() {
        let mut p = params[i];
        p = p.trim_start_matches("!");
        let (idx, _) = program.features_to_idx.get(p).unwrap();
        let feature = program.idx_to_features.get(idx).unwrap();

        if (validation_key & feature.validation_mask()) ^ feature.validation_key() != 0 {
            panic!("Incompatible feature combination \"{}\"", features);
        }

        i += 1;
    }

    return (mask, key, masks, keys);
}

pub(crate) fn parse_features(program: &Program, features: &str) -> (u64, u64) {
    let mut feature = features;
    if feature.starts_with("[") {
        feature = feature.trim_start_matches("[");
    }
    if feature.ends_with("]") {
        feature = feature.trim_end_matches("]");
    }
    
    let params: Vec<&str> = feature.split_whitespace().collect();

    let mut mask: u64 = 0;
    let mut key: u64 = 0;

    let mut validation_key: u64 = 0;

    let mut i: usize = 0;

    while i < params.len() {
        let p = params[i];
        let temp = program.features_to_idx.get(p);
        let (idx, index) = match temp {
            Some(data) => data,
            None => panic!("Unknown feature \"{}\"", p),
        };
        
        let feature = program.idx_to_features.get(idx).unwrap();

        mask |= feature.validation_mask();
        key |= feature.validation_key();


        let offset = 64 - feature.start_byte() - feature.length();
        mask |= ((2 << feature.length() - 1) - 1) << offset;
        key |= (*index as u64) << offset;

        validation_key |= feature.validation_key();

        i += 1;
    }

    i = 0;

    while i < params.len() {
        let p = params[i];
        let (idx, _) = program.features_to_idx.get(p).unwrap();
        let feature = program.idx_to_features.get(idx).unwrap();

        if (validation_key & feature.validation_mask()) ^ feature.validation_key() != 0 {
            panic!("Incompatible feature combination \"{}\"", features);
        }

        i += 1;
    }

    return (mask, key);
}

fn end_feature_def(program: &mut Program) {
    calculate_offsets_recurse(&mut program.features, 0);
    construct_validation_masks(program);
    copy_features_recurse(&mut program.features, &mut program.names_to_idx, &mut program.idx_to_features);
}

fn copy_features_recurse(features: &mut Vec<Feature>, names_to_idx: &mut HashMap<String, u32>, idx_to_features: &mut HashMap<u32, Feature>) {
    let mut i: usize = 0;
    while i < features.len() {
        let feature = &features[i];
        let id = features[i].id();

        names_to_idx.insert(feature.name(), id);
        if idx_to_features.contains_key(&id) {
            let temp = idx_to_features.get_mut(&id).unwrap();
            let remove_mask = !temp.validation_key() ^ feature.validation_key();
            match temp {
                Feature::SwitchType(data) => {
                    data.validation_mask &= remove_mask;
                    data.validation_key &= remove_mask;
                },
                Feature::FeatureDef(data) => {
                    data.validation_mask &= remove_mask;
                    data.validation_key &= remove_mask;
                },
            }
        } else {
            idx_to_features.insert(id, feature.clone_light());
        }

        match feature.clone() {
            Feature::SwitchType(mut data) => {
                let mut j: usize = 0;
                while j < data.features.len() {
                    copy_features_recurse(&mut data.features[j], names_to_idx, idx_to_features);
                    j += 1;
                }
                
            }
            Feature::FeatureDef(_) => {}
        }

        i += 1;
    }
    
}

fn construct_validation_masks(program: &mut Program){
    construct_validation_masks_recurse(&mut program.features, 0, 0);
}

fn construct_validation_masks_recurse(features: &mut Vec<Feature>, current_validation_mask: u64, current_validation_key: u64) {
    let mut i: usize = 0;
    while i < features.len() {
        let f = features[i].clone();
        features[i] = match f {
            Feature::SwitchType(mut data) => {
                data.validation_key = current_validation_key;
                data.validation_mask = current_validation_mask;

                let mut j: usize = 0;
                while j < data.features.len(){
                    let mask = ((2 << data.self_length) - 1) << (64 - data.start_byte - data.self_length);
                    let key = (j as u64 + 1) << (64 - data.start_byte - data.self_length);


                    let temp_validation_key = current_validation_key | key;
                    let temp_validation_mask = current_validation_mask | mask;
    
                    construct_validation_masks_recurse(&mut data.features[j], temp_validation_mask, temp_validation_key);

                    j += 1;
                }

                Feature::SwitchType(data)
            },
            Feature::FeatureDef(mut data) => {
                data.validation_key = current_validation_key;
                data.validation_mask = current_validation_mask;
                Feature::FeatureDef(data)
            },
        };

        i += 1;
    }
}

fn calculate_offsets_recurse(features: &mut Vec<Feature>, offset: u8) {
    let mut i: usize = 0;
    let mut current_offset = offset;
    while i < features.len() {
        let temp = features[i].clone();
        features[i] = match temp {
            Feature::SwitchType(mut data) => {
                data.start_byte = current_offset;
                let mut j: usize = 0;

                while j < data.features.len() {
                    calculate_offsets_recurse(&mut data.features[j], current_offset + data.self_length);
                    j += 1;
                }

                let mut multi_features: HashMap<String, u8> = HashMap::new();

                j = 0;
                while j < data.features.len() {
                    let mut k = 0;
                    while k < data.features[j].len() {
                        let key = data.features[j][k].name();
                        if multi_features.contains_key(&key) {
                            *multi_features.get_mut(&key).unwrap() += 1;
                        } else {
                            multi_features.insert(key, 1);
                        }
                        k += 1;
                    }
                    j += 1;
                }
                
                let mut modified_flag: i32 = 1;
                'outer: while modified_flag > 0 {
                    modified_flag -= 1;

                    j = 0;
                    //test for and get rid of misaligned alike features
                    while j < data.features.len() {

                        let mut k: usize = 0;
                        while k < data.features[j].len() {
                            let mut l = 0;
    
                            let mut collisions: Vec<(usize, usize)> = Vec::new();
                            let mut flag = false;
                            collisions.push((j, k));
                            while l < data.features.len() {
                                if l == j { 
                                    l += 1;
                                    continue; 
                                }
                                let mut m: usize = 0;
                                while m < data.features[l].len() {
                                    if data.features[l][m].name() == data.features[j][k].name() {
                                        collisions.push((l, m));
                                        if data.features[l][m].start_byte() != data.features[j][k].start_byte() {
                                            flag = true;
                                            modified_flag = 2;
                                        }  
                                    }
                                    m += 1;
                                }
                                l += 1;
                            }
                            k += 1;
    
                            
                            if collisions.len() <= 1 { continue; }
                            if !flag { continue; }
                            
                            let mut posses: Vec<u8> = Vec::new();
                            for c in &collisions {
                                posses.push(data.features[c.0][c.1].start_byte());
                            }
    
                            let max_value = *posses.iter().max().unwrap();
                            
                            for c in &collisions {
                                let value = data.features[c.0][c.1].start_byte();
                                if value < max_value {
                                    bump_offsets_recurse(&mut data.features[c.0], c.1, max_value - value);
                                }
                            }
                            }
                        
                        j += 1;
                    }        
                    
                    let mut features_by_start_byte: HashMap<u8, Vec<(usize, usize, String)>> = HashMap::new();
                    j = 0;
                    while j < data.features.len() {
                        let mut k = 0;
                        while k < data.features[j].len() {
                            let start_byte = data.features[j][k].start_byte();
                            let name = data.features[j][k].name();
                            if features_by_start_byte.contains_key(&start_byte) {
                                features_by_start_byte.get_mut(&start_byte).unwrap().push((j, k, name));
                            } else {
                                features_by_start_byte.insert(start_byte, vec!((j, k, name)));
                            }
                            k += 1;
                        }
                        j += 1;
                    }
                
                    j = 0;
                    //test for and get rid of overlapping multi-features
                    while j < data.features.len() {
                        let mut k = 0;
                        while k < data.features[j].len() {
                            for x in &multi_features {
                                let name = data.features[j][k].name();
                                if *x.1 > 1 && *x.0 == name { 
                                    let start_byte = data.features[j][k].start_byte();
                                    let posses = features_by_start_byte.get(&start_byte).unwrap();
                                    if posses.len() > 1 {
                                        let mut flag = false;
                                        for y in posses {
                                            if y.0 != j && y.2 != name {
                                                let amount = data.features[j][k].tot_length();
                                                bump_offsets_recurse(&mut data.features[y.0], y.1, amount);
                                                flag = true;
                                            }
                                        }    
                                        if flag {
                                            modified_flag = 2;
                                            continue 'outer;//to ensure the hash map stays accurate it immediately breaks out to another iteration of the loop
                                        }
                                    }
                                }
                            }
                            k += 1;
                        }
                        j += 1;
                    }
                }
                let mut len: u8 = 0;
                j = 0;
                while j < data.features.len() {
                    let temp = &data.features[j];
                    if temp.len() == 0 {
                        j += 1;
                        continue;
                    }
                    let final_feature = &temp[temp.len() - 1];
                    let this_len = final_feature.start_byte() + final_feature.tot_length() - current_offset;
                    if this_len > len {
                        len = this_len;
                    }
                    j += 1;
                }

                data.tot_length = len;

                current_offset += data.tot_length;
                Feature::SwitchType(data)
            }
            Feature::FeatureDef(mut data) => {
                data.start_byte = current_offset;
                current_offset += data.length;
                Feature::FeatureDef(data)
            }
        };
        i += 1;
    }
}

fn bump_offsets_recurse(features: &mut Vec<Feature>, start_pos: usize, amount: u8) {
    let mut i: usize = start_pos;
    while i < features.len() {
        features[i] = match features[i].clone() {
            Feature::SwitchType(mut data) => {
                data.start_byte += amount;
                let mut j: usize = 0;
                while j < data.features.len() {
                    bump_offsets_recurse(&mut data.features[j], 0, amount);
                    j += 1;
                }
                Feature::SwitchType(data)
            },
            Feature::FeatureDef(mut data) => {
                data.start_byte += amount;
                Feature::FeatureDef(data)
            },
        };
        i += 1;
    }
}

fn construct_switch_line(program: &mut Program, line: &Vec<&str>) -> std::result::Result<(), ConstructorError> {
    if line.len() != 3 {
        return Err(ConstructorError::MalformedDefinition(String::from("Malformed feature definition"), String::from(""), 0));
    }

    let parameter_array: Vec<&str> = line[1].split("(").collect();
    let name = parameter_array[0];
    let mut temp = parameter_array[1].chars();
    temp.next_back();
    let params: Vec<&str> = temp.as_str().split(",").collect();

    let mut option_names: Vec<String> = Vec::new();
    let mut features: Vec<Vec<Feature>> = Vec::new();

    for p in params {
        let temp = p.trim();
        option_names.push(String::from(temp));
        features.push(Vec::new());
    }

    let temp = create_switch_type(String::from(name), option_names, features);

    let mut i: usize = 0;
    while i < temp.option_names.len() {
        program.features_to_idx.insert(temp.option_names[i].clone(), (temp.id, i + 1));
        i += 1;
    }

    assign_feature(program, Feature::SwitchType(temp), line[2])?;
    
    return Ok(());
}

fn construct_feature_def(program: &mut Program, line: &Vec<&str>) -> std::result::Result<(), ConstructorError> {
    if line.len() != 3 {
        return Err(ConstructorError::MalformedDefinition(String::from("Malformed feature definition"), String::from(""), 0));
    }

    if line[1].starts_with("+") {
        let neg = line[1].clone().replace("+", "-");
        let name = line[1].trim_start_matches("+");

        let temp = create_feature_def_bool(String::from(name), neg.clone(), String::from(line[1]));

        program.features_to_idx.insert(String::from(neg), (temp.id, 0));
        program.features_to_idx.insert(String::from(line[1]), (temp.id, 1));

        assign_feature(program, Feature::FeatureDef(temp), line[2])?;
    } else if line[1].ends_with(")") && line[1].contains("(") {
        let parts: Vec<&str> = line[1].split("(").collect();
        if parts.len() != 2 {
            panic!("Malformed feature definition on line \"{}\"", line.join(" "));
        }
        let name = parts[0];
        let feature_names: Vec<&str> = parts[1].trim_end_matches(")").split(",").collect();
        let mut feature_names_final: Vec<String> = Vec::new();

        for n in feature_names {
            feature_names_final.push(String::from(n.trim()));
        }

        let temp =create_feature_def(String::from(name), feature_names_final);

        let mut i: usize = 0;
        while i < temp.option_names.len() {
            program.features_to_idx.insert(temp.option_names[i].clone(), (temp.id, i + 1));
            i += 1;
        }

        assign_feature(program, Feature::FeatureDef(temp), line[2])?;
    } else {
        if line.len() != 3 {
            return Err(ConstructorError::MalformedDefinition(String::from("Malformed feature definition"), String::from(""), 0));
        }
    }
    return Ok(());
}

fn assign_feature(program: &mut Program, feature: Feature, node_name: &str) -> std::result::Result<(), ConstructorError> {
    if node_name == "all" || node_name == "root" {
        program.features.push(feature);
        return Ok(());
    }
    if node_name.starts_with("(") && node_name.ends_with(")") {
        let node_name_mod = node_name.trim_start_matches("(").trim_end_matches(")");
        let nodes: Vec<&str> = node_name_mod.split(",").collect();
        for n in nodes {
            assign_feature_simple(program, feature.clone(), n.trim())?;
        }
    } else {
        assign_feature_simple(program, feature, node_name)?;
    }
    return Ok(());
}

fn assign_feature_simple(program: &mut Program, feature: Feature, node_name: &str) -> std::result::Result<(), ConstructorError> {
    let mut add_count: u8 = 0;
    assign_feature_recurse(&mut program.features, &feature, node_name, &mut add_count);
    if add_count == 0 {
        return Err(ConstructorError::MissingNode(format!("Could not find node {}", node_name), String::from(""), 0));
    }
    Ok(())
}

fn assign_feature_recurse(features: &mut Vec<Feature>, feature: &Feature, node_name: &str, add_count: &mut u8){
    let inverse = node_name.contains("!");
    let search_target: &str = match inverse { true => {let temp: Vec<&str> = node_name.split("!").collect();temp[0]}, false => node_name};
    let mut j: usize = 0;
    while j < features.len() {
        let f = &mut features[j];
        match f {
            Feature::SwitchType(data) => {
                let mut i = 0;
                while i < data.option_names.len() {
                    if data.option_names[i] == search_target {
                        if inverse {
                            let temp: Vec<&str> = node_name.split("!").collect();
                            assign_feature_inverse(&mut data.features[i], feature, temp[1], add_count);
                            return;
                        } else {
                            data.features[i].push(feature.clone());
                            *add_count += 1;
                        }
                    } else {
                        assign_feature_recurse(&mut data.features[i], &feature, node_name, add_count)
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        j += 1;
    }
}

fn assign_feature_inverse(features: &mut Vec<Feature>, feature: &Feature, exception: &str, add_count: &mut u8) {
    let mut i: usize = 0;
    while i < features.len() {
        let f = &mut features[i];
        match f {
            Feature::SwitchType(data) => {
                if data.option_names.contains(&String::from(exception)) {
                    let mut j: usize = 0;
                    while j < data.option_names.len() {
                        if data.option_names[j] != exception {
                            data.features[j].push(feature.clone());
                            *add_count += 1;
                        }
                        j += 1;
                    }
                }
            }
            Feature::FeatureDef(_) => {}
        }
        i += 1;
    }
}