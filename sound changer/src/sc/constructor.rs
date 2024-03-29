use std::{collections::*, vec};

use crate::fancy_regex::Regex;
use {super::applicator::*, super::data::*, super::rules::*};

macro_rules! error {
    ($name:expr, $error_type:expr) => {
        return Err(create_constructor_error_empty($name, line!(), $error_type))
    };
}

macro_rules! error_detail {
    ($name:expr, $error_type:expr, $line_number_user_program:expr, $line_contents:expr) => {
        return Err(create_constructor_error(
            $name,
            $line_contents,
            $line_number_user_program,
            line!(),
            $error_type,
        ))
    };
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum RuleBlockType {
    Rule,
    Sub,
    SubX,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum State {
    None,
    Features,
    Symbols,
    Diacritics,
    Rules,
    RuleAccum(RuleBlockType),
}

pub fn construct(input: &str) -> std::result::Result<Program, ConstructorError> {
    use std::time::Instant;
    let now = Instant::now();

    let mut current_state = State::None;
    let mut program = create_empty_program();
    let mut context = create_program_creation_context();
    let lines: Vec<&str> = input.split('\n').collect();

    let mut rule_accum: Vec<&str> = Vec::new();
    let mut rule_accum_depth: u8 = 0;

    let mut line_number: u32 = 0;
    let regex: Regex = Regex::new(r" (?![^(]*\))(?![^\[]*\])").unwrap();

    for f in lines {
        line_number += 1;

        let line_og = f.clone();
        let mut line = line_og.trim();

        if line.contains('#') {
            let temp: Vec<&str> = line.split('#').collect();
            line = temp[0].trim();
        }

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
                } else if !words[0].is_empty() {
                    error_detail!(
                        format!("Unknown command \"{}\"", words[0]),
                        ConstructorErrorType::UnknownCommandError,
                        line_number,
                        String::from(line_og)
                    );
                }
            }
            State::Features => {
                if words[0] == "switch" {
                    handle_err(
                        construct_switch_line(&mut program, &words),
                        String::from(line_og),
                        line_number,
                    )?;
                } else if words[0] == "feature" {
                    handle_err(
                        construct_feature_def(&mut program, &words),
                        String::from(line_og),
                        line_number,
                    )?;
                } else if words[0] == "end" {
                    handle_err(
                        end_feature_def(&mut program),
                        String::from(line_og),
                        line_number,
                    )?;
                    current_state = State::None;
                } else if !words[0].is_empty() {
                    error_detail!(
                        format!("Unknown command \"{}\"", words[0]),
                        ConstructorErrorType::UnknownCommandError,
                        line_number,
                        String::from(line_og)
                    );
                }
            }
            State::Symbols => {
                if words[0] == "symbol" {
                    handle_err(
                        construct_symbol(&mut program, &words),
                        String::from(line_og),
                        line_number,
                    )?;
                } else if words[0] == "end" {
                    current_state = State::None;
                } else if !words[0].is_empty() {
                    error_detail!(
                        format!("Unknown command \"{}\"", words[0]),
                        ConstructorErrorType::UnknownCommandError,
                        line_number,
                        String::from(line_og)
                    );
                }
            }
            State::Rules => {
                if words[0] != "end" {
                    context
                        .rule_line_defs
                        .insert(program.rules.len(), line_number);
                }
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
                    handle_err(
                        construct_call(&mut program, &words),
                        String::from(line_og),
                        line_number,
                    )?;
                } else if words[0] == "detect" {
                    handle_err(
                        construct_detect(&mut program, &words),
                        String::from(line_og),
                        line_number,
                    )?;
                } else if words[0] == "label" {
                    handle_err(
                        construct_label(&mut program, &words),
                        String::from(line_og),
                        line_number,
                    )?;
                } else if words[0] == "jmp" {
                    handle_err(
                        construct_jump(&mut program, &words),
                        String::from(line_og),
                        line_number,
                    )?;
                } else if words[0] == "end" {
                    check_jumps(&program, &context)?;
                    current_state = State::None;
                } else if !words[0].is_empty() {
                    error_detail!(
                        format!("Unknown command \"{}\"", words[0]),
                        ConstructorErrorType::UnknownCommandError,
                        line_number,
                        String::from(line_og)
                    );
                }
            }
            State::RuleAccum(t) => {
                if words[0] == "rule" {
                    if t == RuleBlockType::Rule {
                        error_detail!(
                            "Malformed rule definition; tried to nest rules",
                            ConstructorErrorType::MalformedDefinition,
                            line_number,
                            String::from(line_og)
                        );
                    } else {
                        rule_accum_depth += 1;
                        rule_accum.push(line);
                    }
                } else if words[0] == "end" {
                    if rule_accum_depth == 1 {
                        match t {
                            RuleBlockType::Rule => handle_err(
                                construct_rule(&mut program, rule_accum),
                                String::from(line_og),
                                line_number,
                            )?,
                            RuleBlockType::Sub => handle_err(
                                construct_sub(&mut program, rule_accum),
                                String::from(line_og),
                                line_number,
                            )?,
                            RuleBlockType::SubX => {
                                handle_err(
                                    construct_sub(&mut program, rule_accum.clone()),
                                    String::from(line_og),
                                    line_number,
                                )?;
                                construct_call(
                                    &mut program,
                                    &vec![
                                        "call",
                                        rule_accum[0].split(' ').collect::<Vec<&str>>()[1],
                                    ],
                                )?; //If it got to this point, bounds are good
                            }
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
            }
            State::Diacritics => {
                if words[0] == "diacritic" {
                    handle_err(
                        construct_diacritic(&mut program, &words),
                        String::from(line_og),
                        line_number,
                    )?;
                } else if words[0] == "end" {
                    current_state = State::None;
                } else if !words[0].is_empty() {
                    error_detail!(
                        format!("Unknown command \"{}\"", words[0]),
                        ConstructorErrorType::UnknownCommandError,
                        line_number,
                        String::from(line_og)
                    );
                }
            }
        }
    }

    match current_state {
        State::None => {}
        State::Features => error_detail!(
            "Features section never finishes",
            ConstructorErrorType::HangingSection,
            line_number,
            String::from("EOF")
        ),
        State::Symbols => error_detail!(
            "Symbols section never finishes",
            ConstructorErrorType::HangingSection,
            line_number,
            String::from("EOF")
        ),
        State::Diacritics => error_detail!(
            "Diacritics section never finishes",
            ConstructorErrorType::HangingSection,
            line_number,
            String::from("EOF")
        ),
        State::Rules => error_detail!(
            "Rules section never finishes",
            ConstructorErrorType::HangingSection,
            line_number,
            String::from("EOF")
        ),
        State::RuleAccum(_) => error_detail!(
            "Rule section never finishes",
            ConstructorErrorType::HangingSection,
            line_number,
            String::from("EOF")
        ),
    }

    let elapsed = now.elapsed();
    println!("Done loading and constructing program in {:.2?}", elapsed);

    Ok(program)
}

/// This function injects more context into the error messages if there is an error, otherwise passes
fn handle_err(
    result: std::result::Result<(), ConstructorError>,
    line: String,
    line_number: u32,
) -> std::result::Result<(), ConstructorError> {
    match result {
        Ok(_) => Ok(()),
        Err(mut v) => {
            v.line_contents = line;
            v.line_number_user_program = match v.line_number_user_program {
                LineNumberInformation::Undetermined => LineNumberInformation::Raw(line_number),
                LineNumberInformation::Offset(v) => {
                    LineNumberInformation::Raw(((line_number as i32) + (v as i32)) as u32)
                }
                LineNumberInformation::Raw(_) => unreachable!(), //Should be unreached; that information is injected here
            };
            Err(v)
        }
    }
}

pub fn construct_words(
    program: &Program,
    input: String,
) -> std::result::Result<Vec<Word>, ApplicationError> {
    let lines: Vec<&str> = input.split('\n').collect();
    let mut result: Vec<Word> = Vec::new();
    for l in lines {
        result.push(from_string(program, &String::from(l.trim()))?);
    }
    Ok(result)
}

fn construct_detect(
    program: &mut Program,
    line: &Vec<&str>,
) -> std::result::Result<(), ConstructorError> {
    if line.len() < 2 {
        error!(
            "Malformed detect definition",
            ConstructorErrorType::MalformedDefinition
        );
    }

    let joined = line.join(" ");
    let rule = joined.trim_start_matches("detect ");
    let split: Vec<&str> = rule.split('/').collect();
    let (predicate, enviorment, inverted) = match split.len() {
        1 => (split[0].trim(), "", false),
        2 => (split[0].trim(), split[1].trim(), false),
        3 => {
            assert_eq!(split[1].trim(), "");
            (split[0].trim(), split[2].trim(), true)
        } //a double slash will split it into 3 sections.
        _ => {
            error!(
                "Malformed rule byte definition",
                ConstructorErrorType::MalformedDefinition
            );
        }
    };

    let (predicate_object, _) = construct_predicate(program, predicate)?;
    let enviorment_object = construct_enviorment(program, enviorment, inverted)?;

    let to_push = create_detect_rule(predicate_object, enviorment_object);
    program.rules.push(to_push);

    Ok(())
}

fn construct_label(
    program: &mut Program,
    line: &Vec<&str>,
) -> std::result::Result<(), ConstructorError> {
    if line.len() != 2 {
        error!(
            "Malformed label definition",
            ConstructorErrorType::MalformedDefinition
        );
    }

    program
        .labels
        .insert(String::from(line[1]), program.rules.len());

    Ok(())
}

fn check_jumps(
    program: &Program,
    context: &ProgramCreationContext,
) -> std::result::Result<(), ConstructorError> {
    //Cannot check if the jump exists on creation, as jumping forward is supported
    //This is called after the rules section to check all the jumps
    let mut i: usize = 0;
    while i < program.rules.len() {
        let rule = &program.rules[i];
        if let Rule::JumpSubRoutine {
            name,
            condition: _,
            inverted: _,
        } = rule
        {
            if !program.labels.contains_key(name) {
                let line_number = context.rule_line_defs.get(&i).unwrap_or(&0u32);
                //Manually guess at line contents instead of finding the actual line contents.
                //Not proper but much easier
                //TODO: Fixme
                error_detail!(
                    format!("Could not find label \"{}\"", name),
                    ConstructorErrorType::MissingLabel,
                    *line_number,
                    format!("jump {}", name)
                );
            }
        }

        i += 1;
    }
    Ok(())
}

fn construct_jump(
    program: &mut Program,
    line: &Vec<&str>,
) -> std::result::Result<(), ConstructorError> {
    if line.len() == 2 {
        program.rules.push(create_jump_rule(
            String::from(line[1]),
            JumpCondition::Unconditional,
            false,
        ));
        return Ok(());
    }
    if line.len() == 3 {
        let mut cond = line[2];
        let flag = if cond.starts_with('!') {
            cond = cond.strip_prefix('!').unwrap();
            true
        } else {
            false
        };

        if cond == "mod" {
            program.rules.push(create_jump_rule(
                String::from(line[1]),
                JumpCondition::PrevMod,
                flag,
            ));
        } else if cond == "flag" {
            program.rules.push(create_jump_rule(
                String::from(line[1]),
                JumpCondition::Flag,
                flag,
            ));
        } else {
            error!(
                format!("Unknown jump condition \"{}\"", cond),
                ConstructorErrorType::UnknownCommandError
            );
        }

        return Ok(());
    }
    error!(
        "Malformed jump definition",
        ConstructorErrorType::MalformedDefinition
    );
}

fn construct_call(
    program: &mut Program,
    line: &Vec<&str>,
) -> std::result::Result<(), ConstructorError> {
    if line.len() != 2 {
        error!(
            "Malformed subroutine call definition",
            ConstructorErrorType::MalformedDefinition
        );
    }

    if program.subroutines.contains_key(line[1]) {
        program
            .rules
            .push(create_subroutine_call_rule(String::from(line[1])));
        Ok(())
    } else {
        error!(
            format!("Could not find subroutine \"{}\"", line[1]),
            ConstructorErrorType::MissingSubroutine
        );
    }
}

fn construct_diacritic(
    program: &mut Program,
    line: &Vec<&str>,
) -> std::result::Result<(), ConstructorError> {
    if line.len() != 5 || line[3] != "=>" {
        error!(
            "Malformed diacritic definition",
            ConstructorErrorType::MalformedDefinition
        );
    }
    let mut symbol = String::from(line[1]);
    symbol.remove_matches("◌");
    let (mask, key) = parse_features_simple(program, line[2])?;
    let (mod_mask, mod_key) = parse_features_simple(program, line[4])?;

    if mask != mod_mask {
        error!(
            "Features don't have the same mask for diacritic",
            ConstructorErrorType::MalformedDefinition
        );
    }

    let diacritic = create_diacritic(symbol, mask, key, mod_key);
    program.diacritics.push(diacritic);
    Ok(())
}

fn construct_sub(
    program: &mut Program,
    lines: Vec<&str>,
) -> std::result::Result<(), ConstructorError> {
    if lines.len() < 2 {
        error!(
            "Malformed subroutine definition",
            ConstructorErrorType::MalformedDefinition
        );
    }
    let line1: Vec<&str> = lines[0].split(' ').collect();
    if line1.len() != 2 {
        error!(
            "Malformed subroutine definition",
            ConstructorErrorType::MalformedDefinition
        );
    }

    let line2: Vec<&str> = lines[1].split(' ').collect();

    if line2.len() != 2 {
        //Single block subroutine
        let to_add = vec![construct_rule_simple(program, lines)?];
        program.subroutines.insert(String::from(line1[1]), to_add);
    } else if line2[0] == "rule" {
        //Multi block subroutine
        let to_add = construct_multi_block_sub(program, lines)?;
        program.subroutines.insert(String::from(line1[1]), to_add);
    } else {
        error!(
            "Malformed subroutine definition",
            ConstructorErrorType::MalformedDefinition
        );
    }

    Ok(())
}

fn construct_multi_block_sub(
    program: &mut Program,
    lines: Vec<&str>,
) -> std::result::Result<Vec<Rule>, ConstructorError> {
    let mut state = State::Rules;
    let mut rule_accum: Vec<&str> = Vec::new();

    let mut flag = true;
    let mut line_number: i8 = 0;
    let mut to_return: Vec<Rule> = Vec::new();
    let regex: Regex = Regex::new(r" (?![^(]*\))(?![^\[]*\])").unwrap();

    for f in &lines {
        if flag {
            //Skip first line
            flag = false;
            continue;
        }
        let mut temp = regex.replace_all(f, String::from_utf8(vec![0]).unwrap());
        let words: Vec<&str> = temp.to_mut().split('\u{0000}').collect();
        line_number += 1;

        match state {
            State::None => panic!(),
            State::Features => panic!(),
            State::Symbols => panic!(),
            State::Diacritics => panic!(),
            State::Rules => {
                if words[0] == "rule" {
                    rule_accum.push(f);
                    state = State::RuleAccum(RuleBlockType::Rule);
                } else if words[0] == "end" {
                    break; //This could cause it to terminate early, except the quantity of ends is tracked when this data is generated
                } else if !words[0].is_empty() {
                    let offset: i8 = line_number - (lines.len() as i8);
                    let mut temp = create_constructor_error_empty(
                        format!("Unknown command \"{}\"", words[0]),
                        line!(),
                        ConstructorErrorType::UnknownCommandError,
                    );
                    temp.line_number_user_program = LineNumberInformation::Offset(offset);
                    return Err(temp);
                }
            }
            State::RuleAccum(_) => {
                if words[0] == "end" {
                    let to_push = match construct_rule_simple(program, rule_accum) {
                        Ok(v) => v,
                        Err(mut v) => {
                            let offset: i8 = line_number - (lines.len() as i8);
                            v.line_number_user_program = match v.line_number_user_program {
                                LineNumberInformation::Undetermined => {
                                    LineNumberInformation::Offset(offset)
                                }
                                LineNumberInformation::Offset(old_offset) => {
                                    LineNumberInformation::Offset(old_offset + offset)
                                }
                                LineNumberInformation::Raw(_) => unreachable!(), //Nothing can give an already completely determined error
                            };
                            return Err(v);
                        }
                    };
                    to_return.push(to_push);
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

fn construct_rule(
    program: &mut Program,
    line: Vec<&str>,
) -> std::result::Result<(), ConstructorError> {
    let temp = construct_rule_simple(program, line)?;
    program.rules.push(temp);
    Ok(())
}

fn construct_rule_simple(
    program: &mut Program,
    line: Vec<&str>,
) -> std::result::Result<Rule, ConstructorError> {
    if line.len() < 2 {
        error!(
            "Malformed rule definition",
            ConstructorErrorType::MalformedDefinition
        );
    }
    let (name, flags) = match construct_rule_header(line[0]) {
        Ok(v) => v,
        Err(mut v) => {
            let offset: i8 = -(line.len() as i8);
            v.line_number_user_program = LineNumberInformation::Offset(offset);
            return Err(v);
        }
    };

    let mut i: usize = 1;
    let mut rule_bytes: Vec<RuleByte> = Vec::new();
    while i < line.len() {
        match construct_rule_byte(program, line[i]) {
            Ok(v1) => {
                if let Some(v2) = v1 {
                    rule_bytes.push(v2)
                }
            }
            Err(mut v) => {
                //The error message is attributed to the end of the statement by default, that is the end statement.
                //This injects an offset to attribute it to the correct line.
                let offset: i8 = (i as i8) - (line.len() as i8);
                v.line_number_user_program = LineNumberInformation::Offset(offset);
                return Err(v);
            }
        }
        i += 1;
    }

    Ok(create_transformation_rule(name, rule_bytes, flags))
}

fn construct_rule_byte(
    program: &Program,
    data: &str,
) -> std::result::Result<Option<RuleByte>, ConstructorError> {
    if data.is_empty() {
        return Ok(None);
    }
    let split1: Vec<&str> = data.split("=>").collect();
    if split1.len() != 2 {
        error!(
            "Malformed rule byte definition",
            ConstructorErrorType::MalformedDefinition
        );
    }

    let split2: Vec<&str> = split1[1].split('/').collect();

    let predicate = split1[0].trim();
    let (result, enviorment, inverted) = match split2.len() {
        1 => (split2[0].trim(), "", false),
        2 => (split2[0].trim(), split2[1].trim(), false),
        3 => {
            assert_eq!(split2[1].trim(), "");
            (split2[0].trim(), split2[2].trim(), true)
        } //a double slash will split it into 3 sections.
        _ => {
            error!(
                "Malformed rule byte definition",
                ConstructorErrorType::MalformedDefinition
            );
        }
    };

    let regex = Regex::new(r" (?![^(]*\))(?![^\[]*\])(?![^\{]*\})").unwrap();
    let mut temp = regex.replace_all(predicate, String::from_utf8(vec![0]).unwrap());
    let predicate_split: Vec<&str> = temp.to_mut().split('\u{0000}').collect();
    let mut temp = regex.replace_all(result, String::from_utf8(vec![0]).unwrap());
    let result_split: Vec<&str> = temp.to_mut().split('\u{0000}').collect();

    if predicate_split.len() != result_split.len() {
        error!(
            "Predicate and result have a different number of elements on rule",
            ConstructorErrorType::MalformedDefinition
        )
    }

    if predicate_split.len() > 1 {
        let mut i: usize = 0;
        let mut predicates: Vec<PredicateDef> = Vec::new();
        let mut results: Vec<ResultDef> = Vec::new();
        while i < predicate_split.len() {
            predicates.push(construct_predicate(program, predicate_split[i])?);
            results.push(construct_result(program, result_split[i])?);
            i += 1;
        }

        //Make same captures have matching masks.
        //Avoids tedious rewriting
        i = 0;
        while i < predicates.len() {
            if i == 0 {
                i += 1;
                continue;
            }
            let mut j = 0;
            while j < predicates[i].1.len() {
                let mut k = 0;
                while k < i {
                    let mut l = 0;
                    while l < predicates[k].1.len() {
                        if predicates[k].1[l].0 == predicates[k].1[j].0 {
                            //Matching predicates
                            predicates[i].1[j].1 = predicates[k].1[l].1;
                        }
                        l += 1;
                    }
                    k += 1;
                }
                j += 1;
            }
            i += 1;
        }

        Ok(Some(create_multi_rule_byte(
            predicates,
            results,
            construct_enviorment(program, enviorment, inverted)?,
        )?))
    } else {
        Ok(Some(create_rule_byte(
            construct_predicate(program, predicate)?,
            construct_result(program, result)?,
            construct_enviorment(program, enviorment, inverted)?,
        )?))
    }
}

fn construct_predicate(
    program: &Program,
    predicate: &str,
) -> std::result::Result<PredicateDef, ConstructorError> {
    let mut input = predicate.trim();

    let mut captures: Vec<(usize, u64)> = Vec::new();
    if input.contains('$') {
        let temp: Vec<&str> = input.split('$').collect();
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
    if input.starts_with('{') && input.ends_with('}') {
        input = input.trim_end_matches('}').trim_start_matches('{');
        let results = construct_predicates(program, input)?;
        return Ok((results, captures));
    }
    if input.contains('{') || input.contains('}') {
        error!(
            "Malformed predicate definition",
            ConstructorErrorType::MalformedDefinition
        );
    }
    if input.starts_with('(') && input.ends_with(')') {
        input = input.trim_end_matches(')').trim_start_matches('(');
        let results = construct_predicates(program, input)?;
        let multi_predicate = create_multi_predicate(results, false);
        return Ok((vec![Box::new(multi_predicate)], captures));
    }
    if input.contains('(') || input.contains(')') {
        error!(
            "Malformed predicate definition",
            ConstructorErrorType::MalformedDefinition
        );
    }

    Ok((vec![construct_simple_predicate(program, input)?], captures))
}

fn construct_predicates(
    program: &Program,
    input: &str,
) -> std::result::Result<Vec<Box<dyn Predicate>>, ConstructorError> {
    let regex = Regex::new(r" (?![^(]*\))(?![^\[]*\])(?![^\{]*\})").unwrap();
    let mut temp = regex.replace_all(input, String::from_utf8(vec![0]).unwrap());
    let input_split: Vec<&str> = temp.to_mut().split('\u{0000}').collect();

    let mut results: Vec<Box<dyn Predicate>> = Vec::new();
    for x in input_split {
        let (mut pred, _) = construct_predicate(program, x)?;
        results.append(&mut pred);
    }

    Ok(results)
}

fn construct_capture(
    program: &Program,
    capture: &str,
) -> std::result::Result<(usize, u64), ConstructorError> {
    if capture.contains('(') {
        if !capture.ends_with(')') {
            error!(
                "Malformed capture definition",
                ConstructorErrorType::MalformedDefinition
            );
        }

        let split: Vec<&str> = capture.split('(').collect();

        let features = split[1].trim_end_matches(')');
        let feature_names: Vec<&str> = features.split(' ').collect();
        let mut mask: u64 = 0;

        for x in feature_names {
            let id = program.names_to_idx.get(x);

            let feature = match id {
                Some(val) => program.idx_to_features.get(val).unwrap(),
                None => {
                    error!(
                        format!("Could not find feature {}", x),
                        ConstructorErrorType::MissingFeature
                    );
                }
            };

            let offset = 64 - feature.start_byte() - feature.length();
            let single_mask = ((2 << (feature.length() - 1)) - 1) << offset;
            mask |= single_mask;
        }

        let temp = split[0].parse::<usize>();
        match temp {
            Ok(val) => Ok((val, mask)),
            Err(_) => {
                error!(
                    format!("Could not read capture id {}", split[0]),
                    ConstructorErrorType::ParseError
                );
            }
        }
    } else {
        let temp = capture.parse::<usize>();
        match temp {
            Ok(val) => Ok((val, 0xFFFFFFFFFFFFFFFF)),
            Err(_) => {
                error!(
                    format!("Could not read capture id {}", capture),
                    ConstructorErrorType::ParseError
                );
            }
        }
    }
}

pub(crate) fn construct_simple_predicate(
    program: &Program,
    predicate: &str,
) -> std::result::Result<Box<dyn Predicate>, ConstructorError> {
    if predicate.starts_with('[') && predicate.ends_with(']') {
        if predicate.contains('!') {
            let (mask, key, masks, keys) = parse_features_negative(program, predicate)?;
            let predicate = create_positive_negative_predicate(mask, key, masks, keys);
            return Ok(Box::new(predicate));
        } else {
            let (mask, key) = parse_features(program, predicate)?;
            let predicate = create_simple_predicate(key, mask);
            return Ok(Box::new(predicate));
        }
    }
    if predicate.contains('[') || predicate.contains(']') {
        error!(
            "Malformed predicate definition",
            ConstructorErrorType::MalformedDefinition
        );
    }

    if program.symbol_to_letter.contains_key(predicate) {
        let (letter, mask) = program.symbol_to_letter.get(predicate).unwrap();
        let predicate = create_simple_predicate(letter.value, *mask);
        return Ok(Box::new(predicate));
    }

    let temp = from_string(program, &String::from(predicate));
    match temp {
        Ok(v) => {
            if v.len() != 1 {
                error!(
                    "Multiple character predicate",
                    ConstructorErrorType::MalformedDefinition
                );
            }
            let predicate = create_simple_predicate(v[0].value, 0xFFFFFFFFFFFFFFFF);
            Ok(Box::new(predicate))
        }
        Err(v) => error!(
            format!("Missing symbol : {}", v.to_string()),
            ConstructorErrorType::MissingSymbol
        ),
    }
}

fn construct_result(
    program: &Program,
    result: &str,
) -> std::result::Result<ResultDef, ConstructorError> {
    let mut input = result.trim();

    let mut captures: Vec<usize> = Vec::new();
    if input.contains('$') {
        let temp: Vec<&str> = input.split('$').collect();
        input = temp[0];
        let mut i: usize = 1;
        while i < temp.len() {
            let (val, _) = construct_capture(program, temp[i].trim())?;
            captures.push(val);
            i += 1;
        }
    }

    if input.starts_with('{') && input.ends_with('}') {
        input = input.trim_end_matches('}').trim_start_matches('{');
        let results = construct_results(program, input)?;
        return Ok((results, captures));
    }
    if input.contains('{') || input.contains('}') {
        error!(
            "Malformed result definition",
            ConstructorErrorType::MalformedDefinition
        );
    }

    Ok((vec![construct_single_result(program, input)?], captures))
}

fn construct_results(
    program: &Program,
    input: &str,
) -> std::result::Result<Vec<Box<dyn Result>>, ConstructorError> {
    let regex = Regex::new(r" (?![^(]*\))(?![^\[]*\])(?![^\{]*\})").unwrap();
    let mut temp = regex.replace_all(input, String::from_utf8(vec![0]).unwrap());
    let input_split: Vec<&str> = temp.to_mut().split('\u{0000}').collect();

    let mut results: Vec<Box<dyn Result>> = Vec::new();
    for x in input_split {
        let (mut pred, _) = construct_result(program, x)?;
        results.append(&mut pred);
    }

    Ok(results)
}

fn construct_single_result(
    program: &Program,
    result: &str,
) -> std::result::Result<Box<dyn Result>, ConstructorError> {
    if result.starts_with(">[") || result.starts_with('[') && result.ends_with(']') {
        if result.starts_with('>') {
            let temp = result.trim_start_matches('>');
            let (_, value) = parse_features(program, temp)?;
            let result = create_simple_result(Letter { value });
            return Ok(Box::new(result));
        } else {
            let (mask, value) = parse_features_simple(program, result)?;
            let result = create_simple_application_result(mask, value);
            return Ok(Box::new(result));
        }
    }
    if result.contains('>') || result.contains('[') || result.contains(']') {
        error!(
            "Malformed result definition",
            ConstructorErrorType::MalformedDefinition
        );
    }

    if result == "*" {
        return Ok(Box::new(create_delete_result()));
    }

    let temp = from_string(program, &String::from(result));
    match temp {
        Ok(v) => {
            if v.len() != 1 {
                error!(
                    "Multiple character result",
                    ConstructorErrorType::MalformedDefinition
                );
            }
            let result = create_simple_result(v[0]);
            Ok(Box::new(result))
        }
        Err(v) => error!(
            format!("Couldn't find symbol: {}", v.to_string()),
            ConstructorErrorType::MissingSymbol
        ),
    }
}

fn parse_features_simple(
    program: &Program,
    features: &str,
) -> std::result::Result<(u64, u64), ConstructorError> {
    let mut feature = features;
    if feature.starts_with('[') {
        feature = feature.trim_start_matches('[');
    }
    if feature.ends_with(']') {
        feature = feature.trim_end_matches(']');
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
            None => {
                error!(
                    format!("Could not find feature {}", params[i]),
                    ConstructorErrorType::MissingFeature
                );
            }
        };

        let feature = program.idx_to_features.get(idx).unwrap();

        let offset = 64 - feature.start_byte() - feature.length();
        mask |= ((2 << (feature.length() - 1)) - 1) << offset;
        key |= (*index as u64) << offset;

        i += 1;
    }

    Ok((mask, key))
}

fn construct_enviorment(
    program: &Program,
    enviorment: &str,
    inverted: bool,
) -> std::result::Result<Enviorment, ConstructorError> {
    if enviorment.is_empty() {
        return Ok(create_empty_enviorment());
    }
    if !enviorment.contains('_') {
        error!(
            "Malformed enviorment definition",
            ConstructorErrorType::MalformedDefinition
        );
    }

    let enviorment_wings: Vec<&str> = enviorment.split('_').collect();
    if enviorment_wings.len() != 2 {
        error!(
            "Malformed enviorment definition",
            ConstructorErrorType::MalformedDefinition
        );
    }

    let (ante_wing, ante_boundary) =
        construct_enviorment_wing(program, enviorment_wings[0], Ordering::Reverse)?;
    let (post_wing, post_boundary) =
        construct_enviorment_wing(program, enviorment_wings[1], Ordering::Forward)?;

    Ok(create_enviorment(
        ante_wing,
        post_wing,
        ante_boundary,
        post_boundary,
        inverted,
    ))
}

fn construct_enviorment_wing(
    program: &Program,
    enviorment: &str,
    direction: Ordering,
) -> std::result::Result<(Vec<EnviormentPredicate>, bool), ConstructorError> {
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
        if c.is_empty() {
            continue;
        }
        if c == "$" {
            flag = true;
        } else {
            if flag {
                error!(
                    "Malformed rule definition: Word boundary condition in middle of enviorment",
                    ConstructorErrorType::MalformedDefinition
                );
            }
            result.push(construct_enviorment_predicate(program, c)?);
        }
    }

    Ok((result, flag))
}

fn construct_enviorment_predicate(
    program: &Program,
    predicate: &str,
) -> std::result::Result<EnviormentPredicate, ConstructorError> {
    if predicate.contains('<') {
        let predicate_split: Vec<&str> = predicate.split('<').collect();
        if predicate_split.len() != 2 {
            error!(
                "Malformed enviorment predicate definition",
                ConstructorErrorType::MalformedDefinition
            );
        }
        let predicate_instance = construct_simple_predicate(program, predicate_split[0])?;

        let quantity_spec = predicate_split[1].trim_end_matches('>');
        let quantities: Vec<&str> = quantity_spec.split(':').collect();
        if quantities.len() != 2 {
            error!(
                "Malformed quantity specifier definition",
                ConstructorErrorType::MalformedDefinition
            );
        }

        let quant_min = quantities[0].parse::<u8>();
        let quant_max = quantities[1].parse::<u8>();

        let quant_min_value = match quant_min {
            Ok(val) => val,
            Err(_) => {
                error!(
                    "Malformed quantity specifier definition",
                    ConstructorErrorType::MalformedDefinition
                );
            }
        };
        let quant_max_value = match quant_max {
            Ok(val) => val,
            Err(_) => {
                error!(
                    "Malformed quantity specifier definition",
                    ConstructorErrorType::MalformedDefinition
                );
            }
        };

        return Ok(create_enviorment_predicate(
            predicate_instance,
            quant_min_value,
            quant_max_value,
        ));
    }

    let predicate_features = predicate.trim_end_matches(&['+', '*', '?']);
    let predicate_instance = construct_simple_predicate(program, predicate_features)?;
    if predicate.ends_with('?') {
        return Ok(create_enviorment_predicate(predicate_instance, 0, 1));
    }
    if predicate.ends_with('*') {
        return Ok(create_enviorment_predicate(predicate_instance, 0, 255));
    }
    if predicate.ends_with('+') {
        return Ok(create_enviorment_predicate(predicate_instance, 1, 255));
    }
    Ok(create_enviorment_predicate_single(predicate_instance))
}

fn construct_rule_header(data: &str) -> std::result::Result<(String, u16), ConstructorError> {
    let words: Vec<&str> = data.split_whitespace().collect();

    if words.len() < 2 {
        error!(
            "Malformed rule header definition",
            ConstructorErrorType::MalformedDefinition
        );
    }

    let name = String::from(words[1]);

    if name.contains(['(', ')', '+', '!', '"', ',']) {
        error!(
            "Invald characters in rule",
            ConstructorErrorType::MalformedDefinition
        );
    }

    Ok((name, 0)) //TODO add flags
}

fn construct_symbol(
    program: &mut Program,
    line: &Vec<&str>,
) -> std::result::Result<(), ConstructorError> {
    if line.len() != 3 {
        error!(
            "Malformed symbol definition",
            ConstructorErrorType::MalformedDefinition
        );
    }

    let symbol = line[1];
    if symbol.contains(['(', ')', '+', '!', '"', ',']) {
        error!(
            "Invald characters in symbol",
            ConstructorErrorType::MalformedDefinition
        );
    }

    let (mask, value) = parse_features(program, line[2])?;
    let letter = Letter { value };
    program
        .letter_to_symbol
        .insert(letter, String::from(symbol));
    program
        .symbol_to_letter
        .insert(String::from(symbol), (letter, mask));
    Ok(())
}

fn parse_features_negative(
    program: &Program,
    features: &str,
) -> std::result::Result<(u64, u64, Vec<u64>, Vec<u64>), ConstructorError> {
    let mut feature = features;
    if feature.starts_with('[') {
        feature = feature.trim_start_matches('[');
    }
    if feature.ends_with(']') {
        feature = feature.trim_end_matches(']');
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
        let flag = p.starts_with('!');
        p = p.trim_start_matches('!');
        let temp = program.features_to_idx.get(p);
        let (idx, index) = match temp {
            Some(data) => data,
            None => {
                error!(
                    format!("Could not find feature {}", p),
                    ConstructorErrorType::MissingFeature
                );
            }
        };

        let feature = program.idx_to_features.get(idx).unwrap();

        validation_key |= feature.validation_key();

        let offset = 64 - feature.start_byte() - feature.length();

        if flag {
            masks.push(((2 << (feature.length() - 1)) - 1) << offset);
            keys.push((*index as u64) << offset);
        } else {
            mask |= ((2 << (feature.length() - 1)) - 1) << offset;
            key |= (*index as u64) << offset;

            mask |= feature.validation_mask();
            key |= feature.validation_key();
        }
        i += 1;
    }

    i = 0;

    while i < params.len() {
        let mut p = params[i];
        p = p.trim_start_matches('!');
        let (idx, _) = program.features_to_idx.get(p).unwrap();
        let feature = program.idx_to_features.get(idx).unwrap();

        if (validation_key & feature.validation_mask()) ^ feature.validation_key() != 0 {
            error!(
                "Incompatible feature combination",
                ConstructorErrorType::InvalidFeature
            );
        }

        i += 1;
    }

    Ok((mask, key, masks, keys))
}

pub(crate) fn parse_features(
    program: &Program,
    features: &str,
) -> std::result::Result<(u64, u64), ConstructorError> {
    let mut feature = features;
    if feature.starts_with('[') {
        feature = feature.trim_start_matches('[');
    }
    if feature.ends_with(']') {
        feature = feature.trim_end_matches(']');
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
            None => {
                error!(
                    format!("Could not find feature {}", p),
                    ConstructorErrorType::MissingFeature
                );
            }
        };

        let feature = program.idx_to_features.get(idx).unwrap();

        mask |= feature.validation_mask();
        key |= feature.validation_key();

        let offset = 64 - feature.start_byte() - feature.length();
        mask |= ((2 << (feature.length() - 1)) - 1) << offset;
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
            error!(
                "Incompatible feature combination",
                ConstructorErrorType::InvalidFeature
            );
        }

        i += 1;
    }

    Ok((mask, key))
}

fn end_feature_def(program: &mut Program) -> std::result::Result<(), ConstructorError> {
    calculate_offsets_recurse(&mut program.features, 0)?;
    construct_validation_masks(program);
    copy_features_recurse(
        &mut program.features,
        &mut program.names_to_idx,
        &mut program.idx_to_features,
    );
    Ok(())
}

fn copy_features_recurse(
    features: &mut Vec<Feature>,
    names_to_idx: &mut HashMap<String, u32>,
    idx_to_features: &mut HashMap<u32, Feature>,
) {
    let mut i: usize = 0;
    while i < features.len() {
        let feature = &features[i];
        let id = features[i].id();

        names_to_idx.insert(feature.name(), id);
        if let std::collections::hash_map::Entry::Vacant(e) = idx_to_features.entry(id) {
            e.insert(feature.clone_light());
        } else {
            let temp = idx_to_features.get_mut(&id).unwrap();
            let remove_mask = !temp.validation_key() ^ feature.validation_key();
            match temp {
                Feature::SwitchType(data) => {
                    data.validation_mask &= remove_mask;
                    data.validation_key &= remove_mask;
                }
                Feature::FeatureDef(data) => {
                    data.validation_mask &= remove_mask;
                    data.validation_key &= remove_mask;
                }
            }
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

fn construct_validation_masks_recurse(
    features: &mut Vec<Feature>,
    current_validation_mask: u64,
    current_validation_key: u64,
) {
    let mut i: usize = 0;
    while i < features.len() {
        let f = features[i].clone();
        features[i] = match f {
            Feature::SwitchType(mut data) => {
                data.validation_key = current_validation_key;
                data.validation_mask = current_validation_mask;

                let mut j: usize = 0;
                while j < data.features.len() {
                    let mask =
                        ((2 << data.self_length) - 1) << (64 - data.start_byte - data.self_length);
                    let key = (j as u64 + 1) << (64 - data.start_byte - data.self_length);

                    let temp_validation_key = current_validation_key | key;
                    let temp_validation_mask = current_validation_mask | mask;

                    construct_validation_masks_recurse(
                        &mut data.features[j],
                        temp_validation_mask,
                        temp_validation_key,
                    );

                    j += 1;
                }

                Feature::SwitchType(data)
            }
            Feature::FeatureDef(mut data) => {
                data.validation_key = current_validation_key;
                data.validation_mask = current_validation_mask;
                Feature::FeatureDef(data)
            }
        };

        i += 1;
    }
}

fn calculate_offsets_recurse(
    features: &mut Vec<Feature>,
    offset: u8,
) -> std::result::Result<(), ConstructorError> {
    let mut i: usize = 0;
    let mut current_offset = offset;
    while i < features.len() {
        let temp = features[i].clone();
        features[i] = match temp {
            Feature::SwitchType(mut data) => {
                data.start_byte = current_offset;
                let mut j: usize = 0;

                while j < data.features.len() {
                    calculate_offsets_recurse(
                        &mut data.features[j],
                        current_offset + data.self_length,
                    )?;
                    j += 1;
                }

                let mut multi_features: HashMap<String, u8> = HashMap::new();

                j = 0;
                while j < data.features.len() {
                    let mut k = 0;
                    while k < data.features[j].len() {
                        let key = data.features[j][k].name();
                        // Clippy creates an error with the automatic suggestion
                        // TODO: Implement manually
                        #[allow(clippy::map_entry)]
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
                                        if data.features[l][m].start_byte()
                                            != data.features[j][k].start_byte()
                                        {
                                            flag = true;
                                            modified_flag = 2;
                                        }
                                    }
                                    m += 1;
                                }
                                l += 1;
                            }
                            k += 1;

                            if collisions.len() <= 1 {
                                continue;
                            }
                            if !flag {
                                continue;
                            }

                            let mut posses: Vec<u8> = Vec::new();
                            for c in &collisions {
                                posses.push(data.features[c.0][c.1].start_byte());
                            }

                            let max_value = *posses.iter().max().unwrap();

                            for c in &collisions {
                                let value = data.features[c.0][c.1].start_byte();
                                if value < max_value {
                                    bump_offsets_recurse(
                                        &mut data.features[c.0],
                                        c.1,
                                        max_value - value,
                                    )?;
                                }
                            }
                        }

                        j += 1;
                    }

                    let mut features_by_start_byte: HashMap<u8, Vec<(usize, usize, String)>> =
                        HashMap::new();
                    j = 0;
                    while j < data.features.len() {
                        let mut k = 0;
                        while k < data.features[j].len() {
                            let start_byte = data.features[j][k].start_byte();
                            let name = data.features[j][k].name();
                            if let std::collections::hash_map::Entry::Vacant(e) =
                                features_by_start_byte.entry(start_byte)
                            {
                                e.insert(vec![(j, k, name)]);
                            } else {
                                features_by_start_byte
                                    .get_mut(&start_byte)
                                    .unwrap()
                                    .push((j, k, name));
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
                                                bump_offsets_recurse(
                                                    &mut data.features[y.0],
                                                    y.1,
                                                    amount,
                                                )?;
                                                flag = true;
                                            }
                                        }
                                        if flag {
                                            modified_flag = 2;
                                            continue 'outer; //to ensure the hash map stays accurate it immediately breaks out to another iteration of the loop
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
                    if temp.is_empty() {
                        j += 1;
                        continue;
                    }
                    let final_feature = &temp[temp.len() - 1];
                    let this_len =
                        final_feature.start_byte() + final_feature.tot_length() - current_offset;
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

fn bump_offsets_recurse(
    features: &mut Vec<Feature>,
    start_pos: usize,
    amount: u8,
) -> std::result::Result<(), ConstructorError> {
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
            }
            Feature::FeatureDef(mut data) => {
                data.start_byte += amount;
                if data.start_byte + data.length >= 64 {
                    error!(
                        "Couldn't bump feature",
                        ConstructorErrorType::FeatureOverflow
                    );
                }
                Feature::FeatureDef(data)
            }
        };
        i += 1;
    }
    Ok(())
}

fn construct_switch_line(
    program: &mut Program,
    line: &Vec<&str>,
) -> std::result::Result<(), ConstructorError> {
    if line.len() != 3 {
        error!(
            "Malformed feature definition",
            ConstructorErrorType::MalformedDefinition
        );
    }

    let parameter_array: Vec<&str> = line[1].split('(').collect();
    let name = parameter_array[0];
    let mut temp = parameter_array[1].chars();
    temp.next_back();
    let params: Vec<&str> = temp.as_str().split(',').collect();

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
        program
            .features_to_idx
            .insert(temp.option_names[i].clone(), (temp.id, i + 1));
        i += 1;
    }

    assign_feature(program, Feature::SwitchType(temp), line[2])?;

    Ok(())
}

fn construct_feature_def(
    program: &mut Program,
    line: &Vec<&str>,
) -> std::result::Result<(), ConstructorError> {
    if line.len() != 3 {
        error!(
            "Malformed feature definition",
            ConstructorErrorType::MalformedDefinition
        );
    }

    if line[1].starts_with('+') {
        let neg = line[1].clone().replace('+', "-");
        let name = line[1].trim_start_matches('+');

        let temp = create_feature_def_bool(String::from(name), neg.clone(), String::from(line[1]));

        program.features_to_idx.insert(neg, (temp.id, 0));
        program
            .features_to_idx
            .insert(String::from(line[1]), (temp.id, 1));

        assign_feature(program, Feature::FeatureDef(temp), line[2])?;
    } else if line[1].ends_with(')') && line[1].contains('(') {
        let parts: Vec<&str> = line[1].split('(').collect();
        if parts.len() != 2 {
            error!(
                "Malformed feature definition",
                ConstructorErrorType::MalformedDefinition
            );
        }
        let name = parts[0];
        let feature_names: Vec<&str> = parts[1].trim_end_matches(')').split(',').collect();
        let mut feature_names_final: Vec<String> = Vec::new();

        for n in feature_names {
            feature_names_final.push(String::from(n.trim()));
        }

        let temp = create_feature_def(String::from(name), feature_names_final);

        let mut i: usize = 0;
        while i < temp.option_names.len() {
            program
                .features_to_idx
                .insert(temp.option_names[i].clone(), (temp.id, i + 1));
            i += 1;
        }

        assign_feature(program, Feature::FeatureDef(temp), line[2])?;
    } else if line.len() != 3 {
        error!(
            "Malformed feature definition",
            ConstructorErrorType::MalformedDefinition
        );
    }
    Ok(())
}

fn assign_feature(
    program: &mut Program,
    feature: Feature,
    node_name: &str,
) -> std::result::Result<(), ConstructorError> {
    if node_name == "all" || node_name == "root" {
        program.features.push(feature);
        return Ok(());
    }
    if node_name.starts_with('(') && node_name.ends_with(')') {
        let node_name_mod = node_name.trim_start_matches('(').trim_end_matches(')');
        let nodes: Vec<&str> = node_name_mod.split(',').collect();
        for n in nodes {
            assign_feature_simple(program, feature.clone(), n.trim())?;
        }
    } else {
        assign_feature_simple(program, feature, node_name)?;
    }
    Ok(())
}

fn assign_feature_simple(
    program: &mut Program,
    feature: Feature,
    node_name: &str,
) -> std::result::Result<(), ConstructorError> {
    let mut add_count: u8 = 0;
    assign_feature_recurse(&mut program.features, &feature, node_name, &mut add_count);
    if add_count == 0 {
        error!(
            format!("Could not find node {}", node_name),
            ConstructorErrorType::MissingNode
        );
    }
    Ok(())
}

fn assign_feature_recurse(
    features: &mut Vec<Feature>,
    feature: &Feature,
    node_name: &str,
    add_count: &mut u8,
) {
    let inverse = node_name.contains('!');
    let search_target: &str = match inverse {
        true => {
            let temp: Vec<&str> = node_name.split('!').collect();
            temp[0]
        }
        false => node_name,
    };
    let mut j: usize = 0;
    while j < features.len() {
        let f = &mut features[j];
        if let Feature::SwitchType(data) = f {
            let mut i = 0;
            while i < data.option_names.len() {
                if data.option_names[i] == search_target {
                    if inverse {
                        let temp: Vec<&str> = node_name.split('!').collect();
                        assign_feature_inverse(&mut data.features[i], feature, temp[1], add_count);
                        return;
                    } else {
                        data.features[i].push(feature.clone());
                        *add_count += 1;
                    }
                } else {
                    assign_feature_recurse(&mut data.features[i], feature, node_name, add_count)
                }
                i += 1;
            }
        }
        j += 1;
    }
}

fn assign_feature_inverse(
    features: &mut Vec<Feature>,
    feature: &Feature,
    exception: &str,
    add_count: &mut u8,
) {
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
