use std::{collections::*, vec};

use {data::*, rules::*, applicator::*};
use super::fancy_regex::Regex;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum RuleBlockType {
    Rule, Sub, SubX
}

enum State {
    None,
    Features,
    Symbols,
    Diacritics,
    Rules,
    RuleAccum(RuleBlockType),
}

pub fn construct(input: &String) -> std::result::Result<Program, ConstructorError> {
    use std::time::Instant;
    let now = Instant::now();


    let mut current_state = State::None;
    let mut program = create_empty_program();
    let lines: Vec<&str> = input.split("\n").collect();

    let mut rule_accum: Vec<&str> = Vec::new();
    let mut rule_accum_depth: u8 = 0;

    let mut line_number: u16 = 0;

    for f in lines {
        line_number += 1;

        let line_og = f.clone();
        let mut line = line_og.trim();

        if line.contains("#") {
            let temp: Vec<&str> = line.split("#").collect();
            line = temp[0];
        }
        
        let regex: Regex = Regex::new(r" (?![^(]*\))(?![^\[]*\])").unwrap();
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
                    return Err(ConstructorError::UnknownCommandError(format!("Unknown command \"{}\"", words[0]), String::from(line_og), line_number, line!()))
                }
            },
            State::Features => {
                if words[0] == "switch" {
                    handle_err(construct_switch_line(&mut program, &words), String::from(line_og), line_number)?;
                } else if words[0] == "feature" {
                    handle_err(construct_feature_def(&mut program, &words), String::from(line_og), line_number)?;
                } else if words[0] == "end" {
                    handle_err(end_feature_def(&mut program), String::from(line_og), line_number)?;
                    current_state = State::None;
                } else if words[0] != "" {
                    return Err(ConstructorError::UnknownCommandError(format!("Unknown command \"{}\"", words[0]), String::from(line_og), line_number, line!()))
                }
            },
            State::Symbols => {
                if words[0] == "symbol" {
                    handle_err(construct_symbol(&mut program, &words), String::from(line_og), line_number)?;
                } else if words[0] == "end" {
                    current_state = State::None;
                } else if words[0] != "" {
                    return Err(ConstructorError::UnknownCommandError(format!("Unknown command \"{}\"", words[0]), String::from(line_og), line_number, line!()))
                }
            },
            State::Rules => {
                if words[0] == "rule" {
                    rule_accum.push(line);
                    current_state = State::RuleAccum(RuleBlockType::Rule);
                    rule_accum_depth = 1;
                } else if words[0] == "subx" {
                    rule_accum.push(line);
                    current_state = State::RuleAccum(RuleBlockType::SubX);
                    rule_accum_depth = 1;
                } else if words[0] == "sub" {
                    rule_accum.push(line);
                    current_state = State::RuleAccum(RuleBlockType::Sub);
                    rule_accum_depth = 1;
                } else if words[0] == "call" {
                    handle_err(construct_call(&mut program, &words), String::from(line_og), line_number)?;
                } else if words[0] == "end" {
                    current_state = State::None;
                } else if words[0] != "" {
                    return Err(ConstructorError::UnknownCommandError(format!("Unknown command \"{}\"", words[0]), String::from(line_og), line_number, line!()))
                }
            },
            State::RuleAccum(t) => {
                if words[0] == "rule" {
                    if t == RuleBlockType::Rule {
                        return Err(ConstructorError::MalformedDefinition(format!("Malformed rule definition; tried to nest rules"), String::from(line_og), line_number, line!()))
                    } else {
                        rule_accum_depth += 1;
                        rule_accum.push(line);
                    }
                } else if words[0] == "end" {
                    if rule_accum_depth == 1 {
                        match t {
                            RuleBlockType::Rule => handle_err(construct_rule(&mut program, rule_accum), String::from(line_og), line_number)?,
                            RuleBlockType::Sub => handle_err(construct_sub(&mut program, rule_accum), String::from(line_og), line_number)?,
                            RuleBlockType::SubX => todo!(),
                        }
                        rule_accum = Vec::new();
                        current_state = State::Rules;
                        rule_accum_depth = 0;
                    } else {
                        rule_accum_depth -= 1;
                        rule_accum.push(line);
                    }
                } else {
                    rule_accum.push(line);
                }
            },
            State::Diacritics => {
                if words[0] == "diacritic" {
                    handle_err(construct_diacritic(&mut program, &words), String::from(line_og), line_number)?;
                } else if words[0] == "end" {
                    current_state = State::None;
                } else if words[0] != "" {
                    return Err(ConstructorError::UnknownCommandError(format!("Unknown command \"{}\"", words[0]), String::from(line_og), line_number, line!()))
                }
            },
        }
    }

    match current_state {
        State::None => {},
        State::Features => return Err(ConstructorError::HangingSection(String::from("Features section never finishes"), String::from("EOF"), line_number, line!())),
        State::Symbols => return Err(ConstructorError::HangingSection(String::from("Symbols section never finishes"), String::from("EOF"), line_number, line!())),
        State::Diacritics => return Err(ConstructorError::HangingSection(String::from("Diacritics section never finishes"), String::from("EOF"), line_number, line!())),
        State::Rules => return Err(ConstructorError::HangingSection(String::from("Rules section never finishes"), String::from("EOF"), line_number, line!())),
        State::RuleAccum(_) => return Err(ConstructorError::HangingSection(String::from("Rule block never finishes"), String::from("EOF"), line_number, line!())),
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
                ConstructorError::UnknownCommandError(m, _, _, n) => ConstructorError::UnknownCommandError(m, line, line_number, n),
                ConstructorError::HangingSection(m, _, _, n) => ConstructorError::HangingSection(m, line, line_number, n),
                ConstructorError::MalformedDefinition(m, _, _, n) => ConstructorError::MalformedDefinition(m, line, line_number, n),
                ConstructorError::MissingNode(m, _, _, n) => ConstructorError::MissingNode(m, line, line_number, n),
                ConstructorError::FeatureOverflow(m, _, _, n) => ConstructorError::FeatureOverflow(m, line, line_number, n),
                ConstructorError::MissingSymbol(m, _, _, n) => ConstructorError::MissingSymbol(m, line, line_number, n),
                ConstructorError::InvalidFeature(m, _, _, n) => ConstructorError::InvalidFeature(m, line, line_number, n),
                ConstructorError::MissingFeature(m, _, _, n) => ConstructorError::MissingFeature(m, line, line_number, n),
                ConstructorError::ParseError(m, _, _, n) => ConstructorError::ParseError(m, line, line_number, n),
                ConstructorError::MissingSubroutine(m, _, _, n) => ConstructorError::MissingSubroutine(m, line, line_number, n),
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

fn construct_call(program: &mut Program, line: &Vec<&str>) -> std::result::Result<(), ConstructorError> {
    if line.len() != 2 {
        return Err(ConstructorError::MalformedDefinition(String::from("Malformed subroutine call definition"), String::from(""), 0, line!()));
    }

    if program.subroutines.contains_key(line[1]) {
        program.rules.push(create_subroutine_call_rule(String::from(line[1])));
        return Ok(());
    } else {
        return Err(ConstructorError::MissingSubroutine(format!("Could not find subroutine \"{}\"", line[1]), String::from(""), 0, line!()));
    }

}

fn construct_diacritic(program: &mut Program, line: &Vec<&str>) -> std::result::Result<(), ConstructorError> {
    if line.len() != 5 || line[3] != "=>" {
        return Err(ConstructorError::MalformedDefinition(String::from("Malformed diacritic definition"), String::from(""), 0, line!()));
    }
    let mut symbol = String::from(line[1]);
    symbol.remove_matches("◌");
    let (mask, key) = parse_features_simple(program, line[2])?;
    let (mod_mask, mod_key) = parse_features_simple(program, line[4])?;

    if mask != mod_mask {
        return Err(ConstructorError::MalformedDefinition(String::from("Features don't have the same mask for diacritic"), String::from(""), 0, line!()));
    }

    let diacritic = create_diacritic(symbol, mask, key, mod_key);
    program.diacritics.push(diacritic);
    Ok(())
}

fn construct_sub(program: &mut Program, lines: Vec<&str>) -> std::result::Result<(), ConstructorError> {
    if lines.len() < 2 {
        return Err(ConstructorError::MalformedDefinition(String::from("Malformed subroutine definition"), String::from(""), 0, line!()));
    }
    let line1: Vec<&str> = lines[0].split(" ").collect();
    if line1.len() != 2 {
        return Err(ConstructorError::MalformedDefinition(String::from("Malformed subroutine definition"), String::from(""), 0, line!()));
    }

    let line2: Vec<&str> = lines[1].split(" ").collect();

    if line2.len() != 2 {
        //Single block subroutine
        let to_add = vec!(construct_rule_simple(program, lines)?);
        program.subroutines.insert(String::from(line1[1]), to_add);
    } else if line2[0] == "rule" {
        //Multi block subroutine
        let to_add = construct_multi_block_sub(program, lines)?;
        program.subroutines.insert(String::from(line1[1]), to_add);
    } else {
        return Err(ConstructorError::MalformedDefinition(String::from("Malformed subroutine definition"), String::from(""), 0, line!()));
    }

    Ok(())
}

fn construct_multi_block_sub(program: &mut Program, lines: Vec<&str>) -> std::result::Result<Vec<Rule>, ConstructorError> {
    let mut state = State::Rules;
    let mut rule_accum: Vec<&str> = Vec::new();

    let mut flag = true;
    let mut to_return: Vec<Rule> = Vec::new();

    for f in lines {
        if flag {//Skip first line
            flag = false;
            continue;
        }
        let regex: Regex = Regex::new(r" (?![^(]*\))(?![^\[]*\])").unwrap();
        let mut temp = regex.replace_all(f, String::from_utf8(vec![0]).unwrap());
        let words: Vec<&str> = temp.to_mut().split('\u{0000}').collect();

        match state {
            State::None => assert!(false),
            State::Features => assert!(false),
            State::Symbols => assert!(false),
            State::Diacritics => assert!(false),
            State::Rules => {
                if words[0] == "rule" {
                    rule_accum.push(f);
                    state = State::RuleAccum(RuleBlockType::Rule);
                } else if words[0] == "end" {
                    break;//This could cause it to terminate early, except the quantity of ends is tracked when this data is generated
                } else if words[0] != "" {
                    return Err(ConstructorError::UnknownCommandError(format!("Unknown command \"{}\"", words[0]), String::from(""), 0, line!()));
                }
            },
            State::RuleAccum(_) => {
                if words[0] == "end" {
                    to_return.push(construct_rule_simple(program, rule_accum)?);
                    rule_accum = Vec::new();
                    state = State::Rules;
                } else {
                    rule_accum.push(f);
                }
            }
        }
    }

    Ok(to_return)
}

fn construct_rule(program: &mut Program, line: Vec<&str>) -> std::result::Result<(), ConstructorError> {
    let temp = construct_rule_simple(program, line)?;
    program.rules.push(temp);
    Ok(())
}

fn construct_rule_simple(program: &mut Program, line: Vec<&str>) -> std::result::Result<Rule, ConstructorError> {
    if line.len() < 2 {
        return Err(ConstructorError::MalformedDefinition(String::from("Malformed rule definition"), String::from(""), 0, line!()));
    }
    let (name, flags) = construct_rule_header(line[0])?;

    let mut i: usize = 1;
    let mut rule_bytes: Vec<RuleByte> = Vec::new();
    while i < line.len() {
        rule_bytes.push(construct_rule_byte(program, line[i])?);
        i += 1;
    }

    Ok(create_transformation_rule(name, rule_bytes, flags))
}

fn construct_rule_byte(program: &Program, data: &str) -> std::result::Result<RuleByte, ConstructorError> {
    let split1: Vec<&str> = data.split("=>").collect();
    if split1.len() != 2 {
        return Err(ConstructorError::MalformedDefinition(String::from("Malformed rule byte definition"), String::from(""), 0, line!()));
    }

    let split2: Vec<&str> = split1[1].split("/").collect();
    if split2.len() > 2 {
        return Err(ConstructorError::MalformedDefinition(String::from("Malformed rule byte definition"), String::from(""), 0, line!()));
    }

    let predicate = split1[0].trim();
    let (result, enviorment) = match split2.len() {
        1 => { (split2[0].trim(), "") }
        2 => { (split2[0].trim(), split2[1].trim()) }
        _ => {         return Err(ConstructorError::MalformedDefinition(String::from("Malformed rule byte definition"), String::from(""), 0, line!())); }
    };

    let regex = Regex::new(r" (?![^(]*\))(?![^\[]*\])(?![^\{]*\})").unwrap();
    let mut temp = regex.replace_all(predicate, String::from_utf8(vec![0]).unwrap());
    let predicate_split: Vec<&str> = temp.to_mut().split('\u{0000}').collect();
    let mut temp = regex.replace_all(result, String::from_utf8(vec![0]).unwrap());
    let result_split: Vec<&str> = temp.to_mut().split('\u{0000}').collect();

    if predicate_split.len() != result_split.len() {
        return Err(ConstructorError::MalformedDefinition(String::from("Predicate and result have a different number of elements on rule"), String::from(""), 0, line!()));
    }

    if predicate_split.len() > 1 {
        let mut i: usize = 0;
        let mut predicates: Vec<(Vec<Box<dyn Predicate>>, Vec<(usize, u64)>)> = Vec::new();
        let mut results: Vec<(Vec<Box<dyn Result>>, Vec<usize>)> = Vec::new();
        while i < predicate_split.len() {
            predicates.push(construct_predicate(program, predicate_split[i])?);
            results.push(construct_result(program, result_split[i])?);
            i += 1;
        }
        return Ok(create_multi_rule_byte(predicates, results, construct_enviorment(program, enviorment)?));
    } else {
        return Ok(create_rule_byte(construct_predicate(program, predicate)?, construct_result(program, result)?, construct_enviorment(program, enviorment)?));
    }
}

fn construct_predicate(program: &Program, predicate: &str) -> std::result::Result<(Vec<Box<dyn Predicate>>, Vec<(usize, u64)>), ConstructorError> {
    let mut input = predicate.trim();

    let mut captures: Vec<(usize, u64)> = Vec::new();
    if input.contains("$") {
        let temp: Vec<&str> = input.split("$").collect();
        input = temp[0];
        let mut i: usize = 1;
        while i < temp.len() {
            let val = construct_capture(program, temp[i].trim())?;
            captures.push(val);
            i += 1;
        }
    }

    if input == "*" { 
        return Ok((Vec::new(), Vec::new()));
    }
    if input.starts_with("{") && input.ends_with("}") {
        input = input.trim_end_matches("}").trim_start_matches("{");
        let results = construct_predicates(program, input)?;
        return Ok((results, captures));
    }
    if input.contains("{") || input.contains("}") {
        return Err(ConstructorError::MalformedDefinition(String::from("Malformed predicate definition"), String::from(""), 0, line!()));
    }
    if input.starts_with("(") && input.ends_with(")") {
        input = input.trim_end_matches(")").trim_start_matches("(");
        let results = construct_predicates(program, input)?;
        let multi_predicate = create_multi_predicate(results, false);
        return Ok((vec!(Box::new(multi_predicate)), captures));
    }
    if input.contains("(") || input.contains(")") {
        return Err(ConstructorError::MalformedDefinition(String::from("Malformed predicate definition"), String::from(""), 0, line!()));
    }

    return Ok((vec!(construct_simple_predicate(program, input)?), captures));
}

fn construct_predicates(program: &Program, input: &str) -> std::result::Result<Vec<Box<dyn Predicate>>, ConstructorError> {
    let regex = Regex::new(r" (?![^(]*\))(?![^\[]*\])(?![^\{]*\})").unwrap();
    let mut temp = regex.replace_all(input, String::from_utf8(vec![0]).unwrap());
    let input_split: Vec<&str> = temp.to_mut().split('\u{0000}').collect();

    let mut results: Vec<Box<dyn Predicate>> = Vec::new();
    for x in input_split {
        let (mut pred, _) = construct_predicate(program, x)?;
        results.append(&mut pred);
    }

    return Ok(results);
}

fn construct_capture(program: &Program, capture: &str) -> std::result::Result<(usize, u64), ConstructorError> {
    if capture.contains("(") {
        if !capture.ends_with(")") {
            return Err(ConstructorError::MalformedDefinition(String::from("Malformed capture definition"), String::from(""), 0, line!()));
        }

        let split: Vec<&str> = capture.split("(").collect();

        let id = program.names_to_idx.get(split[1].trim_end_matches(")"));

        let feature = match id {
            Some(val) => {
                program.idx_to_features.get(val).unwrap()
            },
            None => { return Err(ConstructorError::MissingFeature(format!("Could not find feature {}", split[1].trim_end_matches(")")), String::from(""), 0, line!())); },
        };

        let offset = 64 - feature.start_byte() - feature.length();
        let mask = ((2 << feature.length() - 1) - 1) << offset;

        let temp = split[0].parse::<usize>();
        match temp {
            Ok(val) => return Ok((val, mask)),
            Err(_) => { return Err(ConstructorError::ParseError(format!("Could not read capture id {}", split[0]), String::from(""), 0, line!())); },
        }
    } else {
        let temp = capture.parse::<usize>();
        match temp {
            Ok(val) => return Ok((val, 0xFFFFFFFFFFFFFFFF)),
            Err(_) => { return Err(ConstructorError::ParseError(format!("Could not read capture id {}", capture), String::from(""), 0, line!())); },
        }
    }
}

pub(crate) fn construct_simple_predicate(program: &Program, predicate: &str) -> std::result::Result<Box<dyn Predicate>, ConstructorError> {
    if predicate.starts_with("[") && predicate.ends_with("]") {
        if predicate.contains("!") {
            let (mask, key, masks, keys) = parse_features_negative(program, predicate)?;
            let predicate = create_positive_negative_predicate(mask, key, masks, keys);
            return Ok(Box::new(predicate));  
        } else {
            let (mask, key) = parse_features(program, predicate)?;
            let predicate = create_simple_predicate(key, mask);
            return Ok(Box::new(predicate));    
        }
    }
    if predicate.contains("[") || predicate.contains("]") {
        return Err(ConstructorError::MalformedDefinition(String::from("Malformed predicate"), String::from(""), 0, line!()));
    }

    if program.symbol_to_letter.contains_key(predicate) {
        let (letter, mask) = program.symbol_to_letter.get(predicate).unwrap();
        let predicate = create_simple_predicate(letter.value, *mask);
        return Ok(Box::new(predicate));
    }
    return Err(ConstructorError::MissingSymbol(String::from("Could not find symbol"), String::from(""), 0, line!()));
}

fn construct_result(program: &Program, result: &str) -> std::result::Result<(Vec<Box<dyn Result>>, Vec<usize>), ConstructorError> {
    let mut input = result.trim();

    let mut captures: Vec<usize> = Vec::new();
    if input.contains("$") {
        let temp: Vec<&str> = input.split("$").collect();
        input = temp[0];
        let mut i: usize = 1;
        while i < temp.len() {
            let (val, _) = construct_capture(program, temp[i].trim())?;
            captures.push(val);
            i += 1;
        }
    }

    if input.starts_with("{") && input.ends_with("}") {
        input = input.trim_end_matches("}").trim_start_matches("{");
        let results = construct_results(program, input)?;
        return Ok((results, captures));
    }
    if input.contains("{") || input.contains("}") {
        return Err(ConstructorError::MalformedDefinition(String::from("Malformed result"), String::from(""), 0, line!()));
    }

    return Ok((vec!(construct_single_result(program, input)?), captures));
}

fn construct_results(program: &Program, input: &str) -> std::result::Result<Vec<Box<dyn Result>>, ConstructorError> {
    let regex = Regex::new(r" (?![^(]*\))(?![^\[]*\])(?![^\{]*\})").unwrap();
    let mut temp = regex.replace_all(input, String::from_utf8(vec![0]).unwrap());
    let input_split: Vec<&str> = temp.to_mut().split('\u{0000}').collect();

    let mut results: Vec<Box<dyn Result>> = Vec::new();
    for x in input_split {
        let (mut pred, _) = construct_result(program, x)?;
        results.append(&mut pred);
    }

    return Ok(results);
}

fn construct_single_result(program: &Program, result: &str) -> std::result::Result<Box<dyn Result>, ConstructorError> {
    if result.starts_with(">[") || result.starts_with("[") && result.ends_with("]") {
        if result.starts_with(">") {
            let temp = result.trim_start_matches(">");
            let (_, value) = parse_features(program, temp)?;
            let result = create_simple_result(Letter { value: value });
            return Ok(Box::new(result));
        } else {
            let (mask, value) = parse_features_simple(program, result)?;
            let result = create_simple_application_result(mask, value);
            return Ok(Box::new(result));
        }
    }
    if result.contains(">") || result.contains("[") || result.contains("]") {
        return Err(ConstructorError::MalformedDefinition(String::from("Malformed result definition"), String::from(""), 0, line!()));
    }

    if result == "*" {
        return Ok(Box::new(create_delete_result()));
    }

    if program.symbol_to_letter.contains_key(result) {
        let (letter, _) = program.symbol_to_letter.get(result).unwrap();
        let result = create_simple_result(letter.clone());
        return Ok(Box::new(result));
    }
    return Err(ConstructorError::MissingSymbol(String::from("Could not find symbol"), String::from(""), 0, line!()));
}

fn parse_features_simple(program: &Program, features: &str) -> std::result::Result<(u64, u64), ConstructorError> {
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
            None => { return Err(ConstructorError::MissingFeature(format!("Could not find feature {}", params[i]), String::from(""), 0, line!())); },
        };
        
        let feature = program.idx_to_features.get(idx).unwrap();

        let offset = 64 - feature.start_byte() - feature.length();
        mask |= ((2 << feature.length() - 1) - 1) << offset;
        key |= (*index as u64) << offset;

        i += 1;
    }

    return Ok((mask, key));
}

fn construct_enviorment(program: &Program, enviorment: &str) -> std::result::Result<Enviorment, ConstructorError> {
    if enviorment == "" {
        return Ok(create_empty_enviorment());
    }
    if !enviorment.contains("_") {
        return Err(ConstructorError::MalformedDefinition(String::from("Malformed enviorment definition"), String::from(""), 0, line!()));
    }

    let enviorment_wings: Vec<&str> = enviorment.split("_").collect();
    if enviorment_wings.len() != 2 {
        return Err(ConstructorError::MalformedDefinition(String::from("Malformed enviorment definition"), String::from(""), 0, line!()));
    }

    let (ante_wing, ante_boundary) = construct_enviorment_wing(program, enviorment_wings[0], Ordering::Reverse)?;
    let (post_wing, post_boundary) = construct_enviorment_wing(program, enviorment_wings[1], Ordering::Forward)?;

    return Ok(create_enviorment(ante_wing, post_wing, ante_boundary, post_boundary));
}

fn construct_enviorment_wing(program: &Program, enviorment: &str, direction: Ordering) -> std::result::Result<(Vec<EnviormentPredicate>, bool), ConstructorError> {
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
                return Err(ConstructorError::MalformedDefinition(String::from("Malformed rule definition: Word boundary condition in middle of enviorment"), String::from(""), 0, line!()));
            }
            result.push(construct_enviorment_predicate(program, c)?);
        }
    }

    return Ok((result, flag));
}

fn construct_enviorment_predicate(program: &Program, predicate: &str) -> std::result::Result<EnviormentPredicate, ConstructorError> {
    if predicate.contains("<") {
        let predicate_split: Vec<&str> = predicate.split("<").collect();
        if predicate_split.len() != 2 {
            return Err(ConstructorError::MalformedDefinition(String::from("Malformed enviorment predicate definition"), String::from(""), 0, line!()));
        }
        let predicate_instance = construct_simple_predicate(program, predicate_split[0])?;

        let quantity_spec = predicate_split[1].trim_end_matches(">");
        let quantities: Vec<&str> = quantity_spec.split(":").collect();
        if quantities.len() != 2 {
            return Err(ConstructorError::MalformedDefinition(String::from("Malformed quantity specifier definition"), String::from(""), 0, line!()));
        }

        let quant_min = quantities[0].parse::<u8>();
        let quant_max = quantities[1].parse::<u8>();

        let quant_min_value = match quant_min {
            Ok(val) => val,
            Err(_) => { return Err(ConstructorError::MalformedDefinition(String::from("Malformed quantity specifier definition"), String::from(""), 0, line!())); },
        };
        let quant_max_value = match quant_max {
            Ok(val) => val,
            Err(_) => { return Err(ConstructorError::MalformedDefinition(String::from("Malformed quantity specifier definition"), String::from(""), 0, line!())); },
        };

        return Ok(create_enviorment_predicate(predicate_instance, quant_min_value, quant_max_value));
    }

    let predicate_features = predicate.trim_end_matches(&['+', '*', '+']);
    let predicate_instance = construct_simple_predicate(program, predicate_features)?;
    if predicate.ends_with("?") {
        return Ok(create_enviorment_predicate(predicate_instance, 0, 1));
    }
    if predicate.ends_with("*") {
        return Ok(create_enviorment_predicate(predicate_instance, 0, 255));
    }
    if predicate.ends_with("+") {
        return Ok(create_enviorment_predicate(predicate_instance, 1, 255));
    }
    return Ok(create_enviorment_predicate_single(predicate_instance));
}

fn construct_rule_header(data: &str) -> std::result::Result<(String, u16), ConstructorError> {
    let words: Vec<&str> = data.split_whitespace().collect();

    if words.len() < 2 {
        return Err(ConstructorError::MalformedDefinition(String::from("Malformed rule header definition"), String::from(""), 0, line!()));
    }

    let name = String::from(words[1]);

    if name.contains(&['(', ')', '+', '!', '"', ',']) {
        return Err(ConstructorError::MalformedDefinition(String::from("Invald characters in rule"), String::from(""), 0, line!()));
    }

    return Ok((name, 0));//TODO add flags
}

fn construct_symbol(program: &mut Program, line: &Vec<&str>) -> std::result::Result<(), ConstructorError> {
    if line.len() != 3 {
        return Err(ConstructorError::MalformedDefinition(String::from("Malformed symbol definition"), String::from(""), 0, line!()));
    }

    let symbol = line[1];
    if symbol.contains(&['(', ')', '+', '!', '"', ',']) {
        return Err(ConstructorError::MalformedDefinition(String::from("Invald characters in symbol"), String::from(""), 0, line!()));
    }

    let (mask, value) =  parse_features(&program, line[2])?;
    let letter = Letter{ value: value };
    program.letter_to_symbol.insert(letter, String::from(symbol));
    program.symbol_to_letter.insert(String::from(symbol), (letter, mask));
    Ok(())
}

fn parse_features_negative(program: &Program, features: &str) -> std::result::Result<(u64, u64, Vec<u64>, Vec<u64>), ConstructorError> {
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
            None =>  { return Err(ConstructorError::MissingFeature(format!("Could not find feature {}", p), String::from(""), 0, line!())); },
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
            return Err(ConstructorError::MalformedDefinition(String::from("Incompatible feature combination"), String::from(""), 0, line!()));
        }

        i += 1;
    }

    return Ok((mask, key, masks, keys));
}

pub(crate) fn parse_features(program: &Program, features: &str) -> std::result::Result<(u64, u64), ConstructorError> {
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
            None => { return Err(ConstructorError::MissingFeature(format!("Could not find feature {}", p), String::from(""), 0, line!())); },
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
            return Err(ConstructorError::InvalidFeature(String::from("Incompatible feature combination"), String::from(""), 0, line!()));
        }

        i += 1;
    }

    return Ok((mask, key));
}

fn end_feature_def(program: &mut Program) -> std::result::Result<(), ConstructorError> {
    calculate_offsets_recurse(&mut program.features, 0)?;
    construct_validation_masks(program);
    copy_features_recurse(&mut program.features, &mut program.names_to_idx, &mut program.idx_to_features);
    Ok(())
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

fn construct_validation_masks(program: &mut Program) {
    construct_validation_masks_recurse(&mut program.features, 0, 0)
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

fn calculate_offsets_recurse(features: &mut Vec<Feature>, offset: u8) -> std::result::Result<(), ConstructorError> {
    let mut i: usize = 0;
    let mut current_offset = offset;
    while i < features.len() {
        let temp = features[i].clone();
        features[i] = match temp {
            Feature::SwitchType(mut data) => {
                data.start_byte = current_offset;
                let mut j: usize = 0;

                while j < data.features.len() {
                    calculate_offsets_recurse(&mut data.features[j], current_offset + data.self_length)?;
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
                                    bump_offsets_recurse(&mut data.features[c.0], c.1, max_value - value)?;
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
                                                bump_offsets_recurse(&mut data.features[y.0], y.1, amount)?;
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
    Ok(())
}

fn bump_offsets_recurse(features: &mut Vec<Feature>, start_pos: usize, amount: u8) -> std::result::Result<(), ConstructorError> {
    let mut i: usize = start_pos;
    while i < features.len() {
        features[i] = match features[i].clone() {
            Feature::SwitchType(mut data) => {
                data.start_byte += amount;
                let mut j: usize = 0;
                while j < data.features.len() {
                    bump_offsets_recurse(&mut data.features[j], 0, amount)?;
                    j += 1;
                }
                Feature::SwitchType(data)
            },
            Feature::FeatureDef(mut data) => {
                data.start_byte += amount;
                if data.start_byte + data.length >= 64 {
                    return Err(ConstructorError::FeatureOverflow(String::from("Couldn't bump feature"), String::from(""), 0, line!()));
                }
                Feature::FeatureDef(data)
            },
        };
        i += 1;
    }
    Ok(())
}

fn construct_switch_line(program: &mut Program, line: &Vec<&str>) -> std::result::Result<(), ConstructorError> {
    if line.len() != 3 {
        return Err(ConstructorError::MalformedDefinition(String::from("Malformed feature definition"), String::from(""), 0, line!()));
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
        return Err(ConstructorError::MalformedDefinition(String::from("Malformed feature definition"), String::from(""), 0, line!()));
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
            return Err(ConstructorError::MalformedDefinition(String::from("Malformed feature definition"), String::from(""), 0, line!()));
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
            return Err(ConstructorError::MalformedDefinition(String::from("Malformed feature definition"), String::from(""), 0, line!()));
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
        return Err(ConstructorError::MissingNode(format!("Could not find node {}", node_name), String::from(""), 0, line!()));
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