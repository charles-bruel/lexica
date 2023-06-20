use std::{collections::VecDeque, rc::Rc};

use crate::manual_ux::{
    generative::{
        data_types::{Keyword, Operator},
        tokenizer::TokenType,
    },
    table::{
        GenerativeTableRowProcedure, TableDataTypeDescriptor, TableDescriptor, TableLoadingError,
        TableRow,
    },
};

use super::{
    execution::{
        ColumnSpecifier, EnumNode, IntNode, OutputNode, RangeNode, RuntimeEnum, StringNode,
        TableSpecifier, UIntNode,
    },
    tokenizer::{self, tokenize, Token},
    GenerativeProgram, GenerativeProgramCompileError,
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum Node {
    Enum(EnumNode),
    UInt(UIntNode),
    Int(IntNode),
    String(StringNode),
    Range(RangeNode),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
enum CurrentParsingContext {
    START,
    BLANK,
    READY,
    AWAITING_FUNCTION,
    AWAITING_PARAMETERS,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
enum TableColumnSpecifier {
    TABLE(TableSpecifier),
    COLUMN(ColumnSpecifier),
    BOTH(TableSpecifier, ColumnSpecifier),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct EnumSpecifier {
    name: String,
    column: ColumnSpecifier,
    table: Option<TableSpecifier>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum UnderspecifiedLiteral {
    String(String),
    Enum(EnumSpecifier),
    Int(i32),
    UInt(u32),
    Number(u32, i32),
    StringOrShortEnum(String, ColumnSpecifier, TableSpecifier),
}

impl UnderspecifiedLiteral {
    fn try_convert_enum(
        self,
        data_type: &TableDataTypeDescriptor,
    ) -> Result<EnumNode, GenerativeProgramCompileError> {
        todo!()
    }

    fn try_convert_int(self) -> Result<IntNode, GenerativeProgramCompileError> {
        todo!()
    }

    fn try_convert_uint(self) -> Result<UIntNode, GenerativeProgramCompileError> {
        todo!()
    }

    fn try_convert_string(self) -> Result<StringNode, GenerativeProgramCompileError> {
        todo!()
    }

    fn try_convert_output_node(
        self,
        data_type: &TableDataTypeDescriptor,
    ) -> Result<OutputNode, GenerativeProgramCompileError> {
        todo!()
    }
}

pub fn parse_generative_table_line(
    descriptor: Rc<TableDescriptor>,
    line: &str,
) -> Result<TableRow, TableLoadingError> {
    let tokens = match tokenize(line.to_string()) {
        Some(v) => v,
        None => return Err(TableLoadingError::Unknown),
    };

    // First we extract each column into a vec of tokens
    // Programs are seperated by `|` tokens, so we just
    // iterate through the list and start a new vector
    // after every `|`.
    let mut token_sets = Vec::new();
    token_sets.push(Vec::new());
    for x in tokens {
        match x.token_type {
            TokenType::Operator(Operator::Pipe) => {
                token_sets.push(Vec::new());
            }
            _ => {
                token_sets.last_mut().unwrap().push(x);
            }
        }
    }

    return Ok(TableRow::UnpopulatedTableRow {
        procedure: Rc::new(convert_error(create_generative_table_row_procedure(
            token_sets,
            descriptor.clone(),
        ))?),
        descriptor: descriptor,
    });
}

fn convert_error<T>(
    result: Result<T, GenerativeProgramCompileError>,
) -> Result<T, TableLoadingError> {
    match result {
        Ok(v) => Ok(v),
        Err(e) => Err(TableLoadingError::GenerativeProgramCompileError(e)),
    }
}

/// This assumes that there are a number of token sets equal to the number
/// of columns, and it will panic if not. That error handling should be
/// dealt with by the caller.
fn create_generative_table_row_procedure(
    token_sets: Vec<Vec<Token>>,
    descriptor: Rc<TableDescriptor>,
) -> Result<GenerativeTableRowProcedure, GenerativeProgramCompileError> {
    let mut result: Vec<GenerativeProgram> = Vec::with_capacity(token_sets.len());
    let mut index = 0;
    for tokens in token_sets {
        result.push(create_generative_program(
            tokens,
            &descriptor.column_descriptors[index].data_type,
        )?);
        index += 1;
    }
    todo!()
}

fn create_generative_program(
    tokens: Vec<Token>,
    output_type: &TableDataTypeDescriptor,
) -> Result<GenerativeProgram, GenerativeProgramCompileError> {
    let mut context = CurrentParsingContext::START;
    let mut previous_node: Option<Node> = None;

    let mut queue = VecDeque::from(tokens);

    while queue.len() > 0 {
        // Queue is garunteed to have elements because of while condition
        let current_token = queue.pop_front().unwrap();

        match context {
            CurrentParsingContext::START => match current_token.token_type {
                TokenType::Operator(Operator::Equals) => context = CurrentParsingContext::BLANK,
                _ => {
                    return Ok(GenerativeProgram {
                        output_node: (create_literal_node(current_token, &mut queue)?)
                            .try_convert_output_node(output_type)?,
                    })
                }
            },
            CurrentParsingContext::BLANK => match current_token.token_type {
                TokenType::Keyword(word) => match word {
                    Keyword::Foreach => todo!(),
                    Keyword::Filter => todo!(),
                    Keyword::Save => todo!(),
                    Keyword::Saved => todo!(),
                    _ => return Err(GenerativeProgramCompileError::SyntaxError),
                },
                TokenType::NumericLiteral => todo!(),
                TokenType::Symbol => todo!(),
                _ => return Err(GenerativeProgramCompileError::SyntaxError),
            },
            CurrentParsingContext::READY => todo!(),
            CurrentParsingContext::AWAITING_FUNCTION => todo!(),
            CurrentParsingContext::AWAITING_PARAMETERS => todo!(),
        }
    }

    todo!()
}

/// Returns underspecified literal node based on the tokens. Assumes the
/// context is correct for a literal, will error if not.
fn create_literal_node(
    current_token: Token,
    other_tokens: &mut VecDeque<Token>,
) -> Result<UnderspecifiedLiteral, GenerativeProgramCompileError> {
    match current_token.token_type {
        // String or enum
        TokenType::Symbol => todo!(),
        // Int or uint or enum
        TokenType::NumericLiteral => {
            // First we check if it's an enum definition with a
            // table-column specifier out the front. We will attempt
            // to form a table-column specifier to do this. Since
            // that operation affects the VecDeque of tokens, we
            // need to clone it first, and only if the operation
            // suceeds do we update the main variable.
            let mut secondary_queue = other_tokens.clone();
            match create_table_column_specifier(current_token.clone(), &mut secondary_queue) {
                Ok(_) => {
                    // We need to apply the changes to the main
                    // queue. Unfortunately the easiest way to do
                    // this is to call the function again on the main
                    // queue.
                    // This operation should return successfully, as it
                    // did previous with clones of the parameters given.
                    let table_column =
                        create_table_column_specifier(current_token, other_tokens).unwrap();

                    todo!()
                }
                Err(_) => {
                    // This is fine; it's not an enum with a
                    // table column specifier, so at this point we can
                    // just treat it like a numeric literal.
                    return create_int_or_uint_literal(current_token);
                }
            }
        }
        // Int
        TokenType::Operator(Operator::Minus) => todo!(),
        // Enum
        TokenType::Operator(Operator::Colon) => {
            // It's an enum with a column specified
            // Or it could be a syntax error
            let column_specifier = create_table_column_specifier(current_token, other_tokens)?;

            todo!()
        }
        _ => return Err(GenerativeProgramCompileError::SyntaxError),
    }
}

fn create_enum_literal(
    contents: String,
    specifier: Option<TableColumnSpecifier>,
    current_column: ColumnSpecifier,
) -> Result<EnumSpecifier, GenerativeProgramCompileError> {
    let column = match specifier {
        Some(TableColumnSpecifier::BOTH(_, v)) | Some(TableColumnSpecifier::COLUMN(v)) => v,
        Some(TableColumnSpecifier::TABLE(_)) => {
            return Err(GenerativeProgramCompileError::OnlySpecifiedTable)
        }
        None => current_column,
    };

    let table = match specifier {
        Some(TableColumnSpecifier::BOTH(v, _)) | Some(TableColumnSpecifier::TABLE(v)) => Some(v),
        _ => None,
    };

    return Ok(EnumSpecifier {
        name: contents,
        column,
        table,
    });
}

/// Creates a table-column specifier (`table:column`, `table:`, or `column:`).
/// Assumes the context is correct for a literal, will error if not.
fn create_table_column_specifier(
    current_token: Token,
    other_tokens: &mut VecDeque<Token>,
) -> Result<TableColumnSpecifier, GenerativeProgramCompileError> {
    match current_token.token_type {
        // Column only
        TokenType::Operator(Operator::Colon) => {
            if other_tokens.len() > 0 {
                let next_token = other_tokens.pop_front().unwrap();
                match next_token.token_type {
                    TokenType::NumericLiteral => match next_token.token_contents.parse::<usize>() {
                        Ok(column_id) => {
                            Ok(TableColumnSpecifier::COLUMN(ColumnSpecifier { column_id }))
                        }
                        Err(error) => Err(GenerativeProgramCompileError::IntParseError(error)),
                    },
                    _ => Err(GenerativeProgramCompileError::SyntaxError),
                }
            } else {
                Err(GenerativeProgramCompileError::SyntaxError)
            }
        }
        // Table only or both
        TokenType::NumericLiteral => match current_token.token_contents.parse::<usize>() {
            Ok(table_id) => {
                if other_tokens.len() > 0 {
                    let middle_token = other_tokens.pop_front().unwrap();
                    match middle_token.token_type {
                        TokenType::Operator(Operator::Colon) => {
                            if other_tokens.len() > 0 {
                                let next_token = other_tokens.pop_front().unwrap();
                                match next_token.token_type {
                                    TokenType::NumericLiteral => {
                                        match next_token.token_contents.parse::<usize>() {
                                            Ok(column_id) => Ok(TableColumnSpecifier::BOTH(
                                                TableSpecifier { table_id },
                                                ColumnSpecifier { column_id },
                                            )),
                                            Err(error) => Err(
                                                GenerativeProgramCompileError::IntParseError(error),
                                            ),
                                        }
                                    }
                                    _ => {
                                        // In this case, the specifier is over so we return what
                                        // we have
                                        Ok(TableColumnSpecifier::TABLE(TableSpecifier { table_id }))
                                    }
                                }
                            } else {
                                // This is probably an error but not our responsibility
                                Ok(TableColumnSpecifier::TABLE(TableSpecifier { table_id }))
                            }
                        }
                        _ => Err(GenerativeProgramCompileError::SyntaxError),
                    }
                } else {
                    Err(GenerativeProgramCompileError::SyntaxError)
                }
            }
            Err(error) => Err(GenerativeProgramCompileError::IntParseError(error)),
        },
        _ => Err(GenerativeProgramCompileError::SyntaxError),
    }
}

/// This function creates an `UnderspecifiedLiteral` given a token with contents
/// containing a number. It does not attempt to do anything other than that and
/// will return an error if given the wrong input type. This is to be used after
/// verifying the type and use of the token.
fn create_int_or_uint_literal(
    token: Token,
) -> Result<UnderspecifiedLiteral, GenerativeProgramCompileError> {
    // We will parse into an i64, and check to ensure the
    // value is within the bounds of u32 and i32, and return
    // the proper type (i.e. 3 billion would just return as
    // a UInt because it is above the bounds of an int which
    // uses i32)
    let value = match token.token_contents.parse::<i64>() {
        Ok(v) => v,
        Err(error) => return Err(GenerativeProgramCompileError::IntParseError(error)),
    };
    if value < 0 {
        // This probably shouldn't happen, as negative numbers should
        // appear as two tokens, but it never hurts to include.

        // TODO: Replace with a proper constant
        // It's a magic number right now because I don't have
        // internet and I don't where rust put its int limit
        // constants
        // i32 min
        if value < -2147483648 {
            // Out of range
            return Err(GenerativeProgramCompileError::IntOutOfRange);
        } else {
            return Ok(UnderspecifiedLiteral::Int(value as i32));
            // Int
        }
    } else {
        // TODO: Replace with a proper constants
        // It's a magic number right now because I don't have
        // internet and I don't where rust put its int limit
        // constants
        // i32 max
        if value > 2147483647 {
            // u32 max
            if value > 4294967295 {
                // Out of range
                return Err(GenerativeProgramCompileError::IntOutOfRange);
            } else {
                // UInt
                return Ok(UnderspecifiedLiteral::UInt(value as u32));
            }
        } else {
            // Int or UInt
            return Ok(UnderspecifiedLiteral::Number(value as u32, value as i32));
        }
    }
}
