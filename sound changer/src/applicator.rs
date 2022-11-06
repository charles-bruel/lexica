use super::data::*;

impl super::data::Program {
    pub fn apply(&self, input: Vec<Letter>) -> std::result::Result<Vec<Letter>, ApplicationError> {
        let mut context: ExecutionContext = create_execution_context(&input);
        let mut instruction_count: u16 = 0;
        while context.instruction_ptr < self.rules.len() {
            self.rules[context.instruction_ptr].apply(&self, &mut context)?;

            if !context.jump_flag {
                context.instruction_ptr += 1;
            }
            context.jump_flag = false;
            instruction_count += 1;

            if instruction_count == u16::MAX {
                return Err(ApplicationError::InternalError(String::from("Infinite loop detected; executed u16::MAX instructions without ending")));
            }
        }

        return Ok(context.result);
    }

    pub fn apply_vec(&self, input: Vec<Vec<Letter>>) -> std::result::Result<Vec<Vec<Letter>>, ApplicationError> {
        use std::time::Instant;
        let now = Instant::now();    

        let mut result: Vec<Vec<Letter>> = Vec::new();
        for v in input {
            result.push(self.apply(v)?);
        }

        let elapsed = now.elapsed();
        print!("Applied sound changes to {1} words in {0:.2?}\n", elapsed, result.len());
    
        return Ok(result);
    }
}

impl super::data::Rule {
    pub fn apply(&self, program: &Program, context: &mut ExecutionContext) -> std::result::Result<(), ApplicationError> {
        match self {
            Rule::TransformationRule { bytes, flags: _, name: _ } => {
                context.flag_flag = false;
                context.mod_flag = false;    

                for rule in bytes {
                    let mut mod_flag: bool = false;
                    context.result = rule.apply(std::mem::replace(&mut context.result, vec!()), &mut mod_flag)?;
                    context.mod_flag = mod_flag;
                    //Replaces with an empty struct to avoid ownership issues. I think this is faster than clone.
                }
                return Ok(());
            },
            Rule::CallSubroutine { name } => {
                context.flag_flag = false;
                context.mod_flag = false;
    
                if program.subroutines.contains_key(name) {
                    let temp_subroutine = program.subroutines.get(name).unwrap();
                    for rule in temp_subroutine {
                        rule.apply(program, context)?;
                    }
                    return Ok(());
                } else {
                    return Err(ApplicationError::InternalError(format!("Subroutine not found: \"{}\"", name)));
                }
            },
            Rule::JumpSubRoutine { name, condition, inverted } => {
                let flag = match condition {
                    JumpCondition::PrevMod => context.mod_flag != *inverted,
                    JumpCondition::Flag => context.flag_flag != *inverted,
                    JumpCondition::Unconditional => true,
                };

                context.flag_flag = false;
                context.mod_flag = false;    

                if flag {
                    if program.labels.contains_key(name) {
                        context.instruction_ptr = *program.labels.get(name).unwrap();
                        context.jump_flag = true;
                    } else {
                        return Err(ApplicationError::InternalError(format!("Could not find label \"{}\"", name)));
                    }
                }

                return Ok(())
            },
        }
    }
}

impl super::data::RuleByte {
    pub fn apply(&self, input: Vec<Letter>, mod_flag: &mut bool) -> std::result::Result<Vec<Letter>, ApplicationError> {
        match self.transformations.len() == 1 {
            true => Ok(self.apply_single(input, mod_flag)?),
            false => Ok(self.apply_multi(input, mod_flag)?),
        }
    }

    fn apply_single(&self, input: Vec<Letter>, mod_flag: &mut bool) -> std::result::Result<Vec<Letter>, ApplicationError> {
        if self.transformations[0].predicate.len() == 0 {
            return Ok(self.apply_empty_predicate(input, mod_flag)?);
        } else {
            return Ok(self.apply_single_simple(input, mod_flag)?);
        }
    }

    fn apply_empty_predicate(&self, input: Vec<Letter>, mod_flag: &mut bool) -> std::result::Result<Vec<Letter>, ApplicationError> {
        let mut result = input;
        let mut i: usize = 0;
        while i < result.len() {
            let flag = self.check_enviorment(&result, i, 0);
            if flag {
                let rule = self.transformations[0].result[0].as_ref();
                let temp = match rule.transform(&Letter { value: 0 }) {//Dummy input; there are better ways to do this
                    Some(v) => v,
                    None => return Err(ApplicationError::InternalError(String::from("Rule returned None"))), 
                };
                result.insert(i + 1, temp);
                *mod_flag = true;
                i += 1;
            }

            i += 1;
        }

        return Ok(result);
    }

    fn apply_single_simple(&self, input: Vec<Letter>, mod_flag: &mut bool) -> std::result::Result<Vec<Letter>, ApplicationError> {
        let mut result = input;
        let mut i: usize = 0;
        while i < result.len() {
            let mut flag = false;

            let mut captures: Vec<u64> = vec![0; self.num_captures];
            let mut masks: Vec<u64> = vec![0; self.num_captures];

            let mut j: usize = 0;
            while j < self.transformations[0].predicate.len() {
                let p = &self.transformations[0].predicate[j];
                let temp = p.as_ref();
                if temp.validate(&result, i) {
                    flag = true;
                    let mut k: usize = 0;
                    while k < self.transformations[0].predicate_captures.len() {
                        let x = self.transformations[0].predicate_captures[k].0;
                        let m = self.transformations[0].predicate_captures[k].1;
                        captures[x] = &result[i].value & m;
                        masks[x] = m;

                        k += 1;
                    }
                    break;
                }
                j += 1;
            }

            let mut i_adjustment: i32 = 0;

            if flag && self.check_enviorment(&result, (i as i32 + i_adjustment) as usize, 1) {
                
                let rule = match self.transformations[0].result.len() {
                    1 => self.transformations[0].result[0].as_ref(),
                    _ => self.transformations[0].result[j].as_ref(),
                };
                let temp = rule.transform(&result[(i as i32 + i_adjustment) as usize]);
                match temp {
                    Some(mut val) => {
                        for x in &self.transformations[0].result_captures {
                            val.value = (val.value & !masks[*x]) | captures[*x];
                        }
                        result[i] = val;
                        *mod_flag = true;
                    },
                    None => {
                        result.remove((i as i32 + i_adjustment) as usize);
                        i_adjustment -= 1;
                        *mod_flag = true;
                    },
                }
            }

            i += 1;
            i = (i as i32 + i_adjustment) as usize;
        }
        return Ok(result);
    }

    fn apply_multi(&self, input: Vec<Letter>, mod_flag: &mut bool) -> std::result::Result<Vec<Letter>, ApplicationError> {
        let num = self.transformations.len();

        let mut result = input;
        if num > result.len() {
            return Ok(result);
        }
        let mut i = 0;

        while i <= result.len() - num {
            let mut idx: Vec<usize> = Vec::new();
            let mut captures: Vec<u64> = vec![0; self.num_captures];
            let mut masks: Vec<u64> = vec![0; self.num_captures];

            let mut j: usize = 0;
            let mut flag2 = true;
            while j < self.transformations.len() {
                let mut k: usize = 0;
                let mut flag = false;
                while k < self.transformations[j].predicate.len() {
                    let p = &self.transformations[j].predicate[k];
                    let temp = p.as_ref();
                    if temp.validate(&result, i + j) { 
                        flag = true;
                        let mut l: usize = 0;
                        while l < self.transformations[j].predicate_captures.len() {
                            let x = self.transformations[j].predicate_captures[l].0;
                            let m = self.transformations[j].predicate_captures[l].1;
                            captures[x] = &result[i + j].value & m;
                            masks[x] = m;

                            l += 1;
                        }
                        idx.push(k);
                        break;
                    }
                    k += 1;
                }
                if !flag { flag2 = false; }
                j += 1;
            }

            let mut i_adjustment: i32 = 0;

            if flag2 && self.check_enviorment(&result, i, num) {
                let mut k: usize = 0;
                while k < self.transformations.len() {
                    let rule = match self.transformations[k].result.len() {
                        1 => self.transformations[k].result[0].as_ref(),
                        _ => self.transformations[k].result[idx[k]].as_ref(),
                    };
                    let temp = rule.transform(&result[((i + k) as i32 + i_adjustment) as usize]);
                    match temp {
                        Some(mut val) => {
                            for x in &self.transformations[k].result_captures {
                                val.value = (val.value & !masks[*x]) | captures[*x];
                            }
                            result[((i + k) as i32 + i_adjustment) as usize] = val;
                            *mod_flag = true;
                        },
                        None => {
                            result.remove(((i + k) as i32 + i_adjustment) as usize);
                            *mod_flag = true;
                            if num > result.len() {
                                return Ok(result);
                            }                    
                            i_adjustment -= 1;
                        },
                    }
                    k += 1;
                }
            }

            i += 1;
            i = (i as i32 + i_adjustment) as usize;
        }
        return Ok(result);
    }

    fn check_enviorment(&self, input: &Vec<Letter>, start_position: usize, length: usize) -> bool {
        let mut j: usize = 0;
        let mut position_ante = start_position;
        let mut position_post = start_position + length;
        let mut accum: u8 = 0;

        if length == 0 {
            position_ante += 1;//Not sure why this is nessecary and that's worrying
        }

        while j < self.enviorment.ante.len() {
            if position_ante == 0 {
                if accum < self.enviorment.ante[j].min_quant || j < self.enviorment.ante.len() - 1 {
                    return self.enviorment.inverted;
                }
                break;
            }
            position_ante -= 1;

            let temp = self.enviorment.ante[j].predicate.validate(input, position_ante);
            accum += if temp {1} else {0};
            if temp {
                if accum == self.enviorment.ante[j].max_quant {
                    j += 1;
                    accum = 0;
                }
            } else {
                if accum < self.enviorment.ante[j].min_quant {
                    return self.enviorment.inverted;
                } else {
                    j += 1;
                    accum = 0;
                    if j < self.enviorment.ante.len() {
                        position_ante += 1;//Need to check it again, but against the next one
                    }
                }
            }
        }
        if self.enviorment.ante_word_boundary && position_ante != 0 {
            return self.enviorment.inverted;
        }

        j = 0;
        let mut flag = true;
        while j < self.enviorment.post.len() {
            if (!flag && position_post >= input.len() - 1) || position_post == input.len() {
                if accum < self.enviorment.post[j].min_quant || j < self.enviorment.post.len() - 1 {
                    return self.enviorment.inverted;
                }
                break;
            }
            if flag {
                flag = false;
            } else {
                position_post += 1;
            }

            let temp = self.enviorment.post[j].predicate.validate(input, position_post);
            accum += if temp {1} else {0};
            if temp {
                if accum == self.enviorment.post[j].max_quant {
                    j += 1;
                    accum = 0;
                }
            } else {
                if accum < self.enviorment.post[j].min_quant {
                    return self.enviorment.inverted;
                } else {
                    j += 1;
                    accum = 0;
                    if j < self.enviorment.post.len() {
                        position_post -= 1;//Need to check it again, but against the next one
                    }
                }
            }
        }
        if self.enviorment.post_word_boundary {
            if length == 0 || self.enviorment.post.len() != 0 {
                if position_post != input.len() - 1 {
                    return self.enviorment.inverted;
                }
            } else {
                if position_post != input.len() {
                    return self.enviorment.inverted;
                }
            }
        }

        return !self.enviorment.inverted;
    }
}


pub fn from_string(program: &Program, input: &String) -> std::result::Result<Vec<Letter>, ApplicationError> {
    let mut string = input.clone();
    let mut result: Vec<Letter> = Vec::new();
    let mut keys: Vec<&str> = Vec::new();
    for k in program.symbol_to_letter.keys() {
        keys.push(k);
    }
    keys.sort_unstable_by(|a, b| b.chars().count().cmp(&a.chars().count()));

    let mut depth: usize = 0;
    let mut flag = false;
    while string.len() > 0 {
        if flag {
            for d in &program.diacritics {
                if string.starts_with(&d.diacritic) {
                    string = String::from(string.strip_prefix(&d.diacritic).unwrap());
                    let i = result.len() - 1;
                    if result[i].value & d.mask != d.key {
                        return Err(ApplicationError::IntoConversionError(format!("Invalid diacritic \"{0}\"", d.diacritic)));
                    }
                    result[i].value = (result[i].value & !d.mask) | d.mod_key;
                }
            }
        }
        for k in &keys {
            if string.starts_with(k) {
                string = String::from(string.strip_prefix(k).unwrap());
                let (letter, _) = program.symbol_to_letter.get(*k).unwrap();
                result.push(*letter);
                flag = true;
            }
        }
        depth += 1;
        if depth > 1024 {
            return Err(ApplicationError::IntoConversionError(format!("Could not convert string \"{0}\", got to \"{1}\"", input, string)));
        }
    }
    return Ok(result);
}