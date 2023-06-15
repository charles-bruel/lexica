// The following code has been taken from Lagomorph
// git@github.com:BigTandy/Lagomorph with permission
// from the author

use fancy_regex::Regex;
use std::{collections::HashMap, rc::Rc, hint::unreachable_unchecked};

use super::{data_types::{PrimitiveDataTypes, NumericLiteralEncoding, Operator, Keyword, StringLiteralEncoding}, error_handling::{CompilationError, Out, CompilationErrorType, OutBuilder}};

/// Stores default settings for tokens, used to reduce verbosity on creation
const BASE_TOKEN: TokenDefinition = TokenDefinition {
    token_type: TokenType::Empty,
    descriptor: "",
    regex: None,
    priority: 0,
    match_mode: MatchMode::Regular,
};

/// Contains every type of token that should be matched in the compilation process
const TOKENS: &[TokenDefinition] = &[
    //Grouping types
    TokenDefinition { token_type: TokenType::OpenGroup(GroupType::Paren),   descriptor: "(", priority: 0, ..BASE_TOKEN },
    TokenDefinition { token_type: TokenType::CloseGroup(GroupType::Paren),  descriptor: ")", priority: 0, ..BASE_TOKEN },
    TokenDefinition { token_type: TokenType::OpenGroup(GroupType::Curly),   descriptor: "{", priority: 0, ..BASE_TOKEN },
    TokenDefinition { token_type: TokenType::CloseGroup(GroupType::Curly),  descriptor: "}", priority: 0, ..BASE_TOKEN },
    TokenDefinition { token_type: TokenType::OpenGroup(GroupType::Square),  descriptor: "[", priority: 0, ..BASE_TOKEN },
    TokenDefinition { token_type: TokenType::CloseGroup(GroupType::Square), descriptor: "]", priority: 0, ..BASE_TOKEN },

    //Operators
    TokenDefinition { token_type: TokenType::Operator(Operator::Arrow),  descriptor: "->", priority: 1, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Operator(Operator::Plus),   descriptor: "+",  priority: 1, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Operator(Operator::Minus),  descriptor: "-",  priority: 1, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Operator(Operator::Star),   descriptor: "*",  priority: 1, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Operator(Operator::Slash),  descriptor: "/",  priority: 1, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Operator(Operator::Equals), descriptor: "=",  priority: 1, ..BASE_TOKEN  },

    TokenDefinition { token_type: TokenType::Operator(Operator::PlusPlus),    descriptor: "++", priority: 1, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Operator(Operator::MinusMinus),  descriptor: "--", priority: 1, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Operator(Operator::StarStar),    descriptor: "**", priority: 1, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Operator(Operator::PlusEquals),  descriptor: "+=", priority: 1, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Operator(Operator::MinusEquals), descriptor: "-=", priority: 1, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Operator(Operator::StarEquals),  descriptor: "*=", priority: 1, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Operator(Operator::SlashEquals), descriptor: "/=", priority: 1, ..BASE_TOKEN  },

    TokenDefinition { token_type: TokenType::Operator(Operator::Less),          descriptor: "<",  priority: 1, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Operator(Operator::Greater),       descriptor: ">",  priority: 1, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Operator(Operator::LessEqual),     descriptor: "<=", priority: 1, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Operator(Operator::GreaterEqual),  descriptor: ">=", priority: 1, ..BASE_TOKEN  },


    TokenDefinition { token_type: TokenType::Operator(Operator::Comma),     descriptor: ",",  priority: 1, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Operator(Operator::SemiColon), descriptor: ";",  priority: 1, ..BASE_TOKEN  },

    TokenDefinition { token_type: TokenType::Operator(Operator::Dollar), descriptor: "$",  priority: 1, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Operator(Operator::Bang),   descriptor: "!",  priority: 1, ..BASE_TOKEN  },

    //Keywords
    TokenDefinition { token_type: TokenType::Keyword(Keyword::Function), descriptor: "sub",      priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Keyword(Keyword::Let),      descriptor: "let",      priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Keyword(Keyword::Mut),      descriptor: "mut",      priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Keyword(Keyword::Const),    descriptor: "const",    priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Keyword(Keyword::For),      descriptor: "for",      priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Keyword(Keyword::While),    descriptor: "while",    priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Keyword(Keyword::If),       descriptor: "if",       priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Keyword(Keyword::Else),     descriptor: "else",     priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Keyword(Keyword::Elif),     descriptor: "elif",     priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Keyword(Keyword::Break),    descriptor: "break",    priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Keyword(Keyword::Continue), descriptor: "continue", priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN  },

    TokenDefinition { token_type: TokenType::Keyword(Keyword::I8),   descriptor: "i8 ",  priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN },
    TokenDefinition { token_type: TokenType::Keyword(Keyword::I16),  descriptor: "i16",  priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN },
    TokenDefinition { token_type: TokenType::Keyword(Keyword::I32),  descriptor: "i32",  priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN },
    TokenDefinition { token_type: TokenType::Keyword(Keyword::I64),  descriptor: "i64",  priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN },
    TokenDefinition { token_type: TokenType::Keyword(Keyword::U8),   descriptor: "u8 ",  priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN },
    TokenDefinition { token_type: TokenType::Keyword(Keyword::U16),  descriptor: "u16",  priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN },
    TokenDefinition { token_type: TokenType::Keyword(Keyword::U32),  descriptor: "u32",  priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN },
    TokenDefinition { token_type: TokenType::Keyword(Keyword::U64),  descriptor: "u64",  priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN },
    TokenDefinition { token_type: TokenType::Keyword(Keyword::F32),  descriptor: "f32",  priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN },
    TokenDefinition { token_type: TokenType::Keyword(Keyword::F64),  descriptor: "f64",  priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN },
    TokenDefinition { token_type: TokenType::Keyword(Keyword::Bool), descriptor: "bool", priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN },

    TokenDefinition { token_type: TokenType::Keyword(Keyword::True),  descriptor: "true",  priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN },
    TokenDefinition { token_type: TokenType::Keyword(Keyword::False), descriptor: "false", priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN },

    TokenDefinition { token_type: TokenType::Keyword(Keyword::Define), descriptor: "define", priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN  },
    TokenDefinition { token_type: TokenType::Keyword(Keyword::Export), descriptor: "export", priority: 2, match_mode: MatchMode::Keyword, ..BASE_TOKEN  },
    //Literals
        //(?!\w) is a lookahead sequence; it checks (but doesn't match) that there is NOT a word character
        //after the regex. 
    TokenDefinition { token_type: TokenType::NumericLiteral(PrimitiveDataTypes::I32, NumericLiteralEncoding::Base), descriptor: r"\d+(?!\w)", priority: 3, match_mode: MatchMode::Regex, ..BASE_TOKEN },

    TokenDefinition { token_type: TokenType::StringLiteral(StringLiteralEncoding::Base), descriptor: "\"",  priority: 3, match_mode: MatchMode::StringLiteral(false), ..BASE_TOKEN },
    TokenDefinition { token_type: TokenType::StringLiteral(StringLiteralEncoding::Raw),  descriptor: "r\"", priority: 3, match_mode: MatchMode::StringLiteral(true),  ..BASE_TOKEN },
    
    //Symbols
        //This is a catch-all for any user defined symbols. We enforce some (but not all) of the symbol rules here
        //[a-z,A-Z]             we have stronger constraints for the beginning of a word, must start with a letter
        //         [\w]*        relaxed constraints for the middle of the word
        //              (?!\w)  lookahead, see keywords section
    TokenDefinition { token_type: TokenType::Symbol, descriptor: r"[a-z,A-Z][\w]*(?!\w)", priority: 4, match_mode: MatchMode::Regex, ..BASE_TOKEN },
    
    //Empty
        //Empty captures all the random extra whitespace lying around
        //Priority is kept tight to reduce empty loop cycles
    TokenDefinition { token_type: TokenType::Empty, descriptor: "", priority: 5, match_mode: MatchMode::Empty, ..BASE_TOKEN },

    //Unknown
        //Unknown normally capture syntax errors and other weird things. If it's unknown, it's a syntax error
    TokenDefinition { token_type: TokenType::Unknown, descriptor: "", priority: 6, match_mode: MatchMode::Unknown, ..BASE_TOKEN },
];

/// This stores a set of tokens by priority and the maximum priority of said
/// tokens
type TokenProgram = (HashMap<u16, Vec<TokenDefinition>>, u16);

/// This prepares a list of token definitions for use tokenizing. This compiles
/// any regular expression tokens, as well as arranging tokens by priority. It
/// also computes the maximum priority of any token in the list
fn compile_tokens(tokens: &[TokenDefinition]) -> Result<TokenProgram, fancy_regex::Error> {
    let mut result = HashMap::new();
    let mut max_priority = 0;
    for x in tokens {
        let mut new_token = x.clone();
        if new_token.match_mode == MatchMode::Regex {
            new_token.regex = Some(Regex::new(x.descriptor)?);
        }
        match result.get_mut(&x.priority) {
            None => {
                let vec = vec![new_token];
                result.insert(x.priority, vec);
            }
            Some(v) => v.push(new_token),
        }
        if x.priority > max_priority {
            max_priority = x.priority;
        }
    }
    return Ok((result, max_priority));
}

/// This function replaces all commented blocks of code with spaces, 1:1 with old characters.
/// This keeps token attributions accurate to the original source file.
/// Source file is included only for error handling purposes
pub fn preprocess(mut string: String, source: Rc<SourceFile>) -> Out<String, CompilationError> {
    //We wont bother constructing an OutBuilder because there is only one possible error

    //We are going to be unsafe. How fun

    //We are trying to replace everything in a comment (that is between a "//" and "\n" or
    //between a "/*" and "*/") with a space (0x20), or a newline (0x0A and 0x0D). If we don't
    //encounter a multi-byte character in comments, all good. Otherwise we need to delete that
    //character afterwards to keep the column attributions correct.

    //This contains the start and end indices of all *formerly* multibyte characters
    let mut multi_byte_indices: Vec<(usize, usize)> = Vec::new();
    //This tracks how many *bytes* of multi byte characters we've seen so far
    let mut num_multi_byte: usize = 0;

    //Tracks the current context
    #[derive(PartialEq)]
    enum Status {
        REGULAR,
        LINE,//Line comment
        BLOCK,//Block comment
    }

    //These are used solely for the purposes of error handling and output
    let mut last_block_comment_start_line_number = 0;
    let mut last_block_comment_start_column_number = 0;
    let mut new_line_flag = false;
    let mut line_number = 0;
    let mut column_number = 0;

    unsafe {
        //SAFETY: We must ensure we output value UTF-8 before we pass on the string
        //We will do this by only overwritnig multi-byte characters will completely
        //single byte characters
        let working_string = string.as_bytes_mut();
        let mut mode = Status::REGULAR;
        let mut i = 0;
        while i < working_string.len() {
            //This means it's a single byte UTF-8 codepoint. Most characters should
            //satsify this constraint. This method maximizes efficiency for western
            //scripts, but for example if many comments are written in Korean, this
            //will be very inefficient
            if working_string[i] & 0b10000000 == 0 {
                //All the characters we care about are single bytes

                //If it's a newline, set the new line flag
                if working_string[i] == 0x0A {
                    new_line_flag = true;
                }

                //This stores whether or not the character should be replaced with whitespace
                let mut flag_modify = false;
                match mode {
                    Status::REGULAR => {
                        //For all multi character searches, we also have to verify we aren't exceeding the length of the array\
                        if i < working_string.len() - 1 {
                            //Detect //
                            if working_string[i] == 0x2F && working_string[i + 1] == 0x2F {
                                mode = Status::LINE;
                                flag_modify = true;
                            }
                            //Detect /*
                            if working_string[i] == 0x2F && working_string[i + 1] == 0x2A {
                                mode = Status::BLOCK;
                                flag_modify = true;
                                last_block_comment_start_column_number = column_number;
                                last_block_comment_start_line_number = line_number;
                            }
                        }
                    },
                    Status::LINE => {
                        //Detect \n
                        if working_string[i] == 0x0A {
                            mode = Status::REGULAR;
                        } else {
                            flag_modify = true;
                        }
                    },
                    Status::BLOCK => {
                        //Detect */
                        if i < working_string.len() - 1 {
                            if working_string[i] == 0x2A && working_string[i + 1] == 0x2F {
                                //In order to also remove the last symbol, we do it here
                                //There is possible a less ugly solution to this
                                working_string[i + 1] = 0x20;
                                mode = Status::REGULAR;
                            }
                        }
                        flag_modify = true;
                    },
                }

                if flag_modify {
                    //We don't modify newline characters
                    if working_string[i] != 0x0A && working_string[i] != 0x0D {
                        //Set to space
                        working_string[i] = 0x20;
                    }
                }
            } else {
                //Now we need to determine how long this is
                let len = if working_string[i] & 0b11100000 == 0b11000000 {
                    2
                } else if working_string[i] & 0b11110000 == 0b11100000 {
                    3
                } else if working_string[i] & 0b11111000 == 0b11110000 {
                    4
                } else {
                    //Assuming we've been given valid UTF-8, this is unreachable
                    if cfg!(debug_assertions) {
                        unreachable!();
                    } else {
                        //No unsafe block because we're already unsafe
                        //but this is another level of unsafety
                        unreachable_unchecked();
                    }
                };
                if mode != Status::REGULAR {
                    //Overwrite with spaces
                    //We need to put a valid single byte UTF-8 in all of those spaces
                    //even if we will delete them later, so we just put spaces in
                    for j in i..i + len {
                        working_string[j] = 0x20;
                    }

                    //We want the number of bytes we're deleting, so we subtract one
                    //to represent the one byte from each multi-byte sequence that we
                    //keep.
                    num_multi_byte += len - 1;

                    multi_byte_indices.push((i, i + len));
                }

                //We want to skip directly to the next codepoint
                i += len - 1;
            }

            i += 1;
            if new_line_flag {
                column_number = 0;
                line_number += 1;
                new_line_flag = false;
            } else {
                column_number += 1;
            }
        }

        //At this point we've replaced the contents of all the comments with spaces
        //However, if some of those comments contained multibyte characters (such 
        //as emoji), there are now extra spaces which we must deal with

        //First we check if this step is even nessecary. Most codebases will probably
        //be entirely ASCII
        let final_result = if num_multi_byte == 0 {
            string
        } else {

            //Now we copy the bytes of this string over to a destination string, skipping the ones we
            //don't want. We preallocate the string for maximum performance. To copy into the string,
            //we first create a vec of bytes then copy into it.
            let mut result: Vec<u8> = Vec::with_capacity(working_string.len() - num_multi_byte);
            
            //This contains the index into the array of byte spans to skip. Those are creating in order
            //so we can save execution time by not searching and instead going through them in order.
            let mut index = 0;
            for i in 0..working_string.len() {
                if i > multi_byte_indices[index].0 && i < multi_byte_indices[index].1 {
                    //We skip the character
                } else {
                    result.push(working_string[i]);
                }

                //If we're beyond the current multi_byte to check, we advance, unless we're at the end
                if i >= multi_byte_indices[index].1 && index < multi_byte_indices.len() - 1 {
                    index += 1;
                }
            }

            //SAFETY: We must ensure result contains valid UTF-8. Result contains valid UTF-8 because
            //it is either unmodified bytes of already valid UTF-8, or is single byte UTF-8 characters
            //overwriting *entire* multi-byte characters
            if cfg!(debug_assertions) {
                match String::from_utf8(result) {
                    Ok(v) => v,
                    //This should be unreachable, but we check in debug mode anyway
                    Err(_) => unreachable!(),
                }
            } else {
                String::from_utf8_unchecked(result)
            }
        };

        //We error if there is an unterminated block comment
        //EOF is just as valid as \n for ending a line comment,
        //so we don't check it
        if mode == Status::BLOCK {
            let attribution = Token { 
                token_type: TokenType::Comment,
                token_contents: String::from("/*"),
                line: last_block_comment_start_line_number,
                column: last_block_comment_start_column_number,
                source_file: source.clone(),
            };
            return Out::err(Some(final_result), vec![CompilationError { error_type: CompilationErrorType::UnterminatedBlockComment, message: "Unterminated block comment", attribution }]);
        }

        return Out::success(final_result);
    }
}

/// This is the internal tokenization function, it takes in an array of token defintions
/// and an input string and returns an array of tokens
fn tokenize_int(tokens: TokenProgram, input: String, file: Rc<SourceFile>) -> Vec<Token> {
    let mut result = Vec::new();
    let mut current_string: &str = &input;
    //Prevent repeated allocations by reusing vector
    let mut working_vec: Vec<(TokenType, usize)> = Vec::new();

    //Keep track of line and column numbers so we can assign errors later
    let mut line_number = 0;
    let mut column_number = 0;

    'outer: while current_string.len() > 0 {
        //We first go by priority. Lower is done first
        for tokens_priority in 0..tokens.1 + 1 {
            match tokens.0.get(&tokens_priority) {
                None => { /* Empty priority, no big deal */ }
                Some(v) => {
                    //Now we find everything that matches for this priority
                    //We want to choose the longest match, greedy matching
                    working_vec.clear();
                    'inner: for token in v {
                        match token.match_mode {
                            MatchMode::Regex => {
                                //Unwrap is OK because the regex has to be populated
                                match token.regex.as_ref().unwrap().find(&current_string) {
                                    Ok(Some(v)) => {
                                        //Found a match with the regex
                                        let (start, end) = {
                                            let range = v.range();
                                            (range.start, range.end)
                                        };
                                        if start == 0 {
                                            //This match, is in fact, the beginning of the word
                                            working_vec.push((token.token_type, end));
                                        }
                                    }
                                    Ok(None) => { /* No match, means it's a different token, no big deal */ }
                                    Err(_) => todo!(), //I'm not sure how a regex would error, but we have to handle this
                                }
                            }
                            MatchMode::Regular | MatchMode::Keyword => {
                                if current_string.len() < token.descriptor.len() {
                                    continue 'inner;
                                }
                                let mut iter_test_case = current_string.chars();
                                let mut iter_reference = token.descriptor.chars();
                                while let Some(v) = iter_reference.next() {
                                    //We can blindly advance the current string iterator because we
                                    //checked the length to be less than the one which we are bounds
                                    //checking above
                                    let test_case_char = iter_test_case.next().unwrap();
                                    if v != test_case_char {
                                        //Doesn't match
                                        continue 'inner;
                                    }
                                }
                                //If we reached here, then all the characters match; we're good
                                //However, if we're in keyword match mode, we must verify the next 
                                //character is not a word character
                                if token.match_mode == MatchMode::Keyword {
                                    let next = iter_test_case.next();
                                    match next {
                                        Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_') => {
                                            continue 'inner;
                                        },
                                        None => { /* EOF also counts as a non-word character */ }
                                        _ => { /* Not none, and not a invalid character, all good */}
                                    }
                                }

                                working_vec.push((token.token_type, token.descriptor.len()));
                            }
                            MatchMode::Empty => {
                                let mut iter = current_string.chars();
                                let mut end = 0;
                                while let Some(v) = iter.next() {
                                    if v.is_whitespace() {
                                        end += 1;
                                    } else {
                                        break;
                                    }
                                }
                                if end > 0 {
                                    working_vec.push((token.token_type, end));
                                }
                            },
                            MatchMode::StringLiteral(raw_mode) => {
                                if current_string.len() < token.descriptor.len() {
                                    continue 'inner;
                                }

                                let mut iter_main = current_string.chars();
                                let mut iter_reference = token.descriptor.chars();
                                let mut length = 0;
                                
                                while let Some(v) = iter_reference.next() {
                                    //We can blindly advance the current string iterator because we
                                    //checked the length to be less than the one which we are bounds
                                    //checking above
                                    let test_case_char = iter_main.next().unwrap();
                                    if v != test_case_char {
                                        //Doesn't match
                                        continue 'inner;
                                    }
                                    length += 1;
                                }

                                //If we got here, the string has started. Now we need to see if it ends
                                let mut escape_flag = false;
                                let mut found_end = false;
                                while let Some(v) = iter_main.next() {
                                    length += 1;

                                    //If we have a raw string, we just search for a quote
                                    if raw_mode {
                                        if v == '"' {
                                            found_end = true;
                                            break;
                                        }
                                    }
                                    //Otherwise we have to find a quote that isn't preceeded by a \
                                    //Unless of course that actually a \\
                                    if !raw_mode {
                                        if escape_flag {
                                            //The escape flag lasts for one character. If we escape
                                            //something that isn't valid, again that'll be picked up
                                            //elsewhere
                                            escape_flag = false;
                                        } else {
                                            //We aren't escaped, so this is the true end of the string
                                            if v == '"' {
                                                found_end = true;
                                                break;
                                            } else if v == '\\' {
                                                escape_flag = true;
                                            }
                                        }
                                    }
                                }

                                if !found_end {
                                    //If we didn't find the string, we ignore it and let the program deal
                                    //with it elsewhere

                                    //This could cause issues of detecting a raw string as a string, if it
                                    //couldn't complete a r", then detected the ", but for a raw string not
                                    //to end there would have to be no quotes between it and the EOF, which
                                    //would also mean the regular string can't end, so this is perfectly safe
                                    continue;
                                }

                                //Send this entire thing as a token
                                working_vec.push((token.token_type, length));
                            },
                            MatchMode::Unknown => {
                                //Always matches. If we get here it's garunteed that there is *something* and
                                //we aren't at the EOF, so this is fine. This is last priority, so we aren't
                                //overwriting anything else
                                working_vec.push((token.token_type, 1));
                            },
                        }
                    }
                    if working_vec.len() >= 1 {
                        //We found just one match of this priority, that's the one
                        //We add it to the token list and reset the string

                        let selected_match = {
                            if working_vec.len() == 0 {
                                working_vec[0]
                            } else {
                                let mut max_length = 0;
                                let mut best_index = 0;
                                //This marks if there are multiple longest solutions
                                let mut flag = false;

                                for i in 0..working_vec.len() {
                                    if working_vec[i].1 > max_length {
                                        max_length = working_vec[i].1;
                                        best_index = i;

                                        //Current longest solution is alone... for now
                                        flag = false;
                                    } else if working_vec[i].1 == max_length {
                                        //Current longest solution now has a friend
                                        flag = true;
                                    } else {
                                        //Not the new longest; ignore
                                    }
                                }

                                if flag {
                                    //Ambigious symbols
                                    todo!();
                                }

                                working_vec[best_index]
                            }
                        };

                        //Don't bother adding empty tokens to the list. They're
                        //there just to handle whitespace and we don't care about
                        //it
                        if selected_match.0 != TokenType::Empty {
                            result.push(Token {
                                token_type: selected_match.0,
                                //This takes the section of the string the match returned
                                token_contents: String::from(&current_string[0..selected_match.1]),
                                line: line_number,
                                column: column_number,
                                //Cloning the reference not the actual contents
                                source_file: file.clone(),
                            });
                        }

                        //We need to update line and column numbers. We could do this mathmatically
                        //for column numbers, but we need to check every character for \n, so we
                        //also do column numbers that way
                        let mut iter = current_string.chars();
                        for _ in 0..selected_match.1 {
                            column_number += 1;
                            match iter.next() {
                                Some(v) => {
                                    if v == '\n' {
                                        column_number = 0;
                                        line_number += 1;
                                    }
                                },
                                //It is safe to panic here becuase it cannot be reached. The end of the
                                //selected match must be within the bounds of the string. A panic here
                                //would indicate the token extends beyond the end of the string which is
                                //impossible
                                None => unreachable!(),
                            }
                        }

                        //This takes a slice of the string starting at the end of the match
                        current_string = &current_string[selected_match.1..];

                        //We don't search through the remaining priorities
                        //Explicit marking of control flow, this returns to the outermost loop
                        continue 'outer;
                    } else {
                        //Nothing matches, no need to worry
                    }
                }
            }
        }

        //If we get to here, we had a full loop cycle with no progress. In 
        //addition, the catch-all unknown hasn't triggered. This should be
        //completely unreachable if an unknown is included in the tokens, which
        //it should be
        if cfg!(debug_assertions) {
            unreachable!();
        } else {
            unsafe { unreachable_unchecked(); }
        }
    }
    return result;
}

/// This function tokenizes an input file. It takes an input file and source file
/// reference (which will be attached to the token attributions) and returns a vec
/// of tokens.
#[allow(unused_variables)]
pub fn tokenize(file: Rc<SourceFile>) -> Out<Vec<Token>, CompilationError> {
    let mut out = OutBuilder::new();

    let input = file.src.clone();
    use std::time::Instant;
    let start = Instant::now();
    let comp_start = Instant::now();

    //If this does not unwrap successfully, that is a program error. The supplied
    //regular expressions must compile for the compiler to be valid. Therefore,
    //panicking here is entirely appropiate
    let compiled_tokens = compile_tokens(TOKENS).unwrap();

    let comp_duration = comp_start.elapsed();
    let pre_start = Instant::now();

    let preproccessed_string = match out.test(preprocess(input, file.clone())) {
        Some(v) => v,

        //Even if there is an error, preprocess should still return sensible output. If
        //not, we give the tokenizer the contents of file originally. That will almost
        //certainly cause problems, but it allows us to resume "gracefully"
        None => file.src.clone(),
    };

    let pre_duration = pre_start.elapsed();
    let tok_start = Instant::now();

    let result = tokenize_int(compiled_tokens, preproccessed_string, file);

    let tok_duration = tok_start.elapsed();
    let duration = start.elapsed();

    // println!("Compiled tokens in {:?}", comp_duration);
    // println!("Pre-proccessed in {:?}", pre_duration);
    // println!("Tokenized in {:?}", tok_duration);
    // println!("Total runtime: {:?}", duration);

    return OutBuilder::out(result, out);
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Token {
    /// Stores the [TokenType], so that for tokens which have limited variants,
    /// such as a keyword, don't have to use token_contents 
    pub token_type: TokenType,
    /// Stores the exact text content of this token, used for symbols and literals
    pub token_contents: String,

    /// Tracks the line number in the file this token is from. If the token
    /// crosses multiple lines, it references the start line
    pub line: u32,
    /// Tracks the column of the line in the file this token is from. It 
    /// references the start column
    pub column: u16,
    /// Tracks a reference the file this token was originally from
    pub source_file: Rc<SourceFile>,//We want to save memory and only track a reference to this source file
}

/// This stores information about the source file a token came from, used for
/// error attribution and other utilities
#[derive(Debug, Eq, PartialEq)]
pub struct SourceFile {
    pub file_name: String,
    pub src: String,
}

/// This stores all the information nessecary to identify a certain token as a
/// token of that type
#[derive(Debug, Clone)]
struct TokenDefinition {
    /// This stores the type of the token which this definition matches
    token_type: TokenType,
    /// This stores what this definition matches in the input. The exact 
    /// behaviour depends on the [MatchMode] in match_mode
    descriptor: &'static str,
    /// If a regex is needed, it is cached and stored here.
    regex: Option<Regex>,
    /// Smaller priority number means it's checked first. This garuntees keywords
    /// get checked before symbols. Beyond the priority stated here, tokens are
    /// secondarily prioritized by length of match
    priority: u16,
    /// The match mode which determines how descriptor is used to match tokens
    match_mode: MatchMode,
}

/// This stores the match mode of a certain token definition. It's used internally
/// to swap between different detecting schemes
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum MatchMode {
    /// This matches the contents of the descriptor exactly
    Regular,
    /// This matches the contents of the descriptor exactly and also ensures the character
    /// following it is not a word character (so that in substring, it does not detect sub
    /// as a keyword)
    Keyword,
    /// This uses a regular expression to match, the descriptor is the regex which will be
    /// used to match
    Regex,
    /// This matches a string literal. It has special behaviour because string literals need
    /// be able to match through pretty much anything, and across lines
    StringLiteral(bool),
    /// This matches whitespace
    Empty,
    /// Matches a single anything. Used for cleaning up syntax errors so that tokenize is
    /// garunteed not to panic
    Unknown,
}

/// This stores the type of token that was tokenized. It stores some information about the
/// contents, such as the keyword, but for symbols and literals, the actual contents need to
/// come from the contents field of the parent Token object
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TokenType {
    /// An empty token, matching whitespace. These are stripped out before being returned
    Empty,
    /// A single character that could not be parsed any other way. This indicates a syntax
    /// error of some kind
    Unknown,
    /// An entire comment block. These are not created except for error handling
    Comment,
    /// A token that opens a grouping section, either a "(", "[", or "{"
    OpenGroup(GroupType),
    /// A token that closes a grouping section, either a ")", "]", or "}"
    CloseGroup(GroupType),
    /// An operator. The word operator is very vague and covers things like "+", "-", as well 
    /// as "->" and even ";", ","
    Operator(Operator),
    /// A keyword, pretty much any reserved word. Also contains the exact keyword
    Keyword(Keyword),
    /// A symbol is a programmer defined identifier for a variable, function, etc. This covers
    /// any word or word-like thing that is not a keyword
    Symbol, //The contents are sent seperate to the type TODO: Maybe make the content come along with this?
    /// A literal is a value, such as "100" in code. Also sometimes called immediates. This stores a number
    NumericLiteral(PrimitiveDataTypes, NumericLiteralEncoding),
    /// A literal is a value, such as "100" in code. Also sometimes called immediates. This stores a string
    StringLiteral(StringLiteralEncoding),
}

/// Stores a grouping type, i.e. {}, [], ()
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GroupType {
    /// {}
    Curly,
    /// ()
    Paren,
    /// []
    Square,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_compile() {
        //Tests that the tokens do compile
        compile_tokens(TOKENS).unwrap();
    }

    #[test]
    fn tokenize_keyword_priority() {
        let temp = tokenize(create_test_src_file("while")).unwrap();
        assert_eq!(temp.len(), 1);
        assert_eq!(temp[0].token_type, TokenType::Keyword(Keyword::While));
    }

    #[test]
    fn tokenize_word_check() {
        let temp = tokenize(create_test_src_file("substring")).unwrap();
        assert_eq!(temp.len(), 1);
        assert_eq!(temp[0].token_type, TokenType::Symbol);
    }

    #[test]
    fn tokenize_attribution_test_1() {
        let temp = tokenize(create_test_src_file("sub foo(bar baz) {")).unwrap();
        assert_eq!(temp.len(), 7);
        assert_eq!(temp[2].column, 7);
    }

    #[test]
    fn tokenize_attribution_test_2() {
        let temp = tokenize(create_test_src_file("sub main() {\nconst u32 limit = 100;")).unwrap();
        assert_eq!(temp.len(), 11);
        assert_eq!(temp[4].line, 0);
        assert_eq!(temp[5].line, 1);
    }

    #[test]
    fn strip_comments_test_1() {
        let result = preprocess(String::from("foo /* foo */"), create_empty_test_src_file()).unwrap();
        assert_eq!(result, "foo          ");
    }

    #[test]
    fn strip_comments_test_2() {
        let result = preprocess(String::from("foo // foo"), create_empty_test_src_file()).unwrap();
        assert_eq!(result, "foo       ");
    }

    #[test]
    fn strip_comments_test_3() {
        let result = preprocess(String::from("foo /* foo"), create_empty_test_src_file()).unwrap_err();
        assert_eq!(result[0].attribution.line, 0);
        assert_eq!(result[0].attribution.column, 4);
        assert_eq!(result[0].attribution.token_contents, "/*");
    }

    #[test]
    fn strip_comments_test_4() {
        let result = preprocess(String::from("//foo\nbar"), create_empty_test_src_file()).unwrap();
        assert_eq!(result, "     \nbar");
    }

    #[test]
    fn strip_comments_test_5() {
        let result = preprocess(String::from("foo/*\n*/"), create_empty_test_src_file()).unwrap();
        assert_eq!(result, "foo  \n  ");
    }

    #[test]
    fn strip_comments_test_6() {
        let result = preprocess(String::from("//foo ī"), create_empty_test_src_file()).unwrap();
        assert_eq!(result, "       ");
    }

    #[test]
    fn strip_comments_test_7() {
        let result = preprocess(String::from("//foo 汉"), create_empty_test_src_file()).unwrap();
        assert_eq!(result, "       ");
    }

    #[test]
    fn strip_comments_test_8() {
        let result = preprocess(String::from("//foo 👀"), create_empty_test_src_file()).unwrap();
        assert_eq!(result, "       ");
    }

    #[test]
    fn strip_comments_test_9() {
        //Testing a flag emoji. It's two codepoints combined, so the output will have
        //two characters where it used to be
        let result = preprocess(String::from("//foo 🇵🇬"), create_empty_test_src_file()).unwrap();
        assert_eq!(result, "        ");
    }

    #[test]
    fn strip_comments_test_10() {
        //Testing skin tone modifiers. The skin tone modifier is a character, so the output
        //will have two characters where it used to be
        let result = preprocess(String::from("//foo 👋🏻"), create_empty_test_src_file()).unwrap();
        assert_eq!(result, "        ");
    }

    #[test]
    fn string_literal_test_1() {
        let temp = tokenize(create_test_src_file("= r\"foo\"")).unwrap();
        assert_eq!(temp.len(), 2);
        assert_eq!(temp[1].token_contents, "r\"foo\"");
    }

    #[test]
    fn string_literal_test_2() {
        let temp = tokenize(create_test_src_file("= \"\\\"foo\\\"\"")).unwrap();
        assert_eq!(temp.len(), 2);
        assert_eq!(temp[1].token_contents, "\"\\\"foo\\\"\"");
    }

    #[test]
    fn string_literal_test_3() {
        let temp = tokenize(create_test_src_file("= \"\\\"foo\\\\\"")).unwrap();
        assert_eq!(temp.len(), 2);
        assert_eq!(temp[1].token_contents, "\"\\\"foo\\\\\"");
    }

    #[test]
    fn string_literal_test_4() {
        let temp = tokenize(create_test_src_file("= \"foo\nfoo\"")).unwrap();
        assert_eq!(temp.len(), 2);
        assert_eq!(temp[1].token_contents, "\"foo\nfoo\"");
    }

    #[test]
    fn string_literal_test_5() {
        let temp = tokenize(create_test_src_file("\"foo\nfoo")).unwrap();
        assert_eq!(temp[0].token_type, TokenType::Unknown);
    }

    #[test]
    fn string_literal_test_6() {
        let temp = tokenize(create_test_src_file("r\"foo\nfoo")).unwrap();
        //The first thing will be parsed as a symol, so we just make sure it wasn't
        //parsed as a string literal first
        assert_ne!(temp[0].token_type, TokenType::StringLiteral(StringLiteralEncoding::Raw));
        assert_eq!(temp[1].token_type, TokenType::Unknown);

    }
    
    #[test]
    fn tokenize_int_test_1() {
        //Tests that the tokenizer outputs the expected types
        //Note: does not check that the contents are correct
        let temp = tokenize(create_test_src_file("sub fib(u32 n) -> u32 {")).unwrap();
        assert_eq!(temp.len(), 9);
        for i in 0..9 {
            assert_eq!(temp[i].token_type, INT_TEST_1_EXPECT_TYPE[i]);
            assert_eq!(temp[i].column, INT_TEST_1_EXPECT_COLUMN[i]);
        }
    }

    fn create_test_src_file(contents: &'static str) -> Rc<SourceFile> {
        Rc::new(SourceFile { file_name: String::from("Test"), src: String::from(contents)})
    }

    fn create_empty_test_src_file() -> Rc<SourceFile> {
        Rc::new(SourceFile { file_name: String::from("Test"), src: String::from("")})
    }

    const INT_TEST_1_EXPECT_TYPE: [TokenType; 9] = [
        TokenType::Keyword(Keyword::Function),
        TokenType::Symbol,
        TokenType::OpenGroup(GroupType::Paren),
        TokenType::Keyword(Keyword::U32),
        TokenType::Symbol,
        TokenType::CloseGroup(GroupType::Paren),
        TokenType::Operator(Operator::Arrow),
        TokenType::Keyword(Keyword::U32),
        TokenType::OpenGroup(GroupType::Curly),
    ];

    const INT_TEST_1_EXPECT_COLUMN: [u16; 9] = [
        0, 4, 7, 8, 12, 13, 15, 18, 22
    ];

}