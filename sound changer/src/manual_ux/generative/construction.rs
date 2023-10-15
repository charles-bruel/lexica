use std::{collections::VecDeque, rc::Rc};

use crate::manual_ux::{
    generative::{
        data_types::{Keyword, Operator},
        tokenizer::{TokenType, GroupType},
        SyntaxErrorType,
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
    GenerativeProgram, GenerativeProgramCompileError, node_builder::BuilderNode,
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum Node {
    // TODO: Also add a way to specify enum definition here
    Enum(EnumNode),
    UInt(UIntNode),
    Int(IntNode),
    String(StringNode),
    Range(RangeNode),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum DataTypeDescriptor {
    TableDataType(TableDataTypeDescriptor),
    TableColumnSpecifier,
    Expression
}

#[allow(non_camel_case_types)]
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum ParsingContext {
    /// Indicates that the parser is awaiting the initial `=` sign
    START,
    /// Indicates there is no relevant context in the parser and that
    /// it should receive a value of the given type.
    /// Occurs after the start state, and after an operator such as `+`
    /// (not `.`). The code associated with this state is also called from
    /// the `AWAITING_PARAMETERS` state.
    AWAITING_VALUE(DataTypeDescriptor),
    /// Indicates that there is a valid construction of the given
    /// data type and the operation can be finished or expanded with
    /// any number of operators or symbols, such as `.` or ` + `
    READY,
    /// Indicates that a `.` symbol was correctly used so a function
    /// call should follow.
    /// The value it stores is the node that precedes the dot, i.e. that
    /// will be passed as the first parameter, or `NONE` if the function
    /// is being called statically.
    AWAITING_FUNCTION(Option<Node>),
    /// Indicates that function keyword was used and we now need the
    /// opening `(` of the function.
    /// The value stored is the eventual parameters of the function
    /// (it will be determined when this is set, so it is transfer
    /// through here to it's final destination).
    AWAITING_FUNCTION_BRACKET(VecDeque<DataTypeDescriptor>),
    /// Indicates that we are in a function and awaiting some number of
    /// parameters, or that the function has concluded and we are awaiting
    /// the closing `)`.
    /// The value it stores is the types of the upcoming parameters;
    /// it is also how the system keeps track of the number of parameters
    /// left.
    AWAITING_PARAMETERS(VecDeque<DataTypeDescriptor>),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub enum TableColumnSpecifier {
    TABLE(TableSpecifier),
    COLUMN(ColumnSpecifier),
    BOTH(TableSpecifier, ColumnSpecifier),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct EnumSpecifier {
    name: String,
    column: ColumnSpecifier,
    table: Option<TableSpecifier>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum UnderspecifiedLiteral {
    String(String),
    Enum(EnumSpecifier),
    Int(i32),
    UInt(u32),
    Number(u32, i32),
    TableColumnSpecifier(TableColumnSpecifier),
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

    fn try_convert_table_column(self) -> Result<TableColumnSpecifier, GenerativeProgramCompileError> {
        todo!()
    }

}

pub fn parse_generative_table_line(
    descriptor: Rc<TableDescriptor>,
    mut line: &str,
) -> Result<TableRow, TableLoadingError> {
    // First we need to extract the useful portion of the line which
    // should be in the form :={<CONTENT>} and tokenize it.
    line = line.trim();
    if &line[0..3] != ":={" || line.chars().nth_back(0).unwrap() != '}' {
        return Err(TableLoadingError::GenerativeProgramCompileError(
            GenerativeProgramCompileError::SyntaxError(SyntaxErrorType::MissingProgramSurrondings),
        ));
    }

    let tokens = match tokenize(line[3..line.len() - 2].to_string()) {
        Some(v) => v,
        None => return Err(TableLoadingError::Unknown),
    };

    // Then we extract each column into a vec of tokens
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
            descriptor.clone(),
        )?);
        index += 1;
    }
    todo!()
}

fn create_generative_program(
    tokens: Vec<Token>,
    output_type: &TableDataTypeDescriptor,
    descriptor: Rc<TableDescriptor>
) -> Result<GenerativeProgram, GenerativeProgramCompileError> {
    let main_segment = parse_generative_segment(tokens, output_type, descriptor)?;

    Ok(GenerativeProgram {
        output_node: main_segment.try_convert_output_node(&DataTypeDescriptor::TableDataType(output_type.clone()))?,
    })
}

fn parse_generative_segment(
    tokens: Vec<Token>,
    output_type: &TableDataTypeDescriptor,
    descriptor: Rc<TableDescriptor>
) -> Result<BuilderNode, GenerativeProgramCompileError> {
    let mut context = ParsingContext::START;

    let mut queue = VecDeque::from(tokens);

    let mut main_node: Option<BuilderNode> = None;

    while queue.len() > 0 {
        // Queue is garunteed to have elements because of while condition
        let current_token = queue.pop_front().unwrap();

        println!("{:?}", current_token);

        match context {
            // TODO: Split into own function
            ParsingContext::START => match current_token.token_type {
                TokenType::Operator(Operator::Equals) => {
                    context = ParsingContext::AWAITING_VALUE(DataTypeDescriptor::TableDataType(output_type.clone()))
                }
                _ => {
                    return Ok(
                        BuilderNode::GenericLiteral(create_literal_node(current_token, &mut queue, descriptor.clone())?)
                    )
                }
            },
            ParsingContext::AWAITING_VALUE(ref target_data_type) => {
                let mut clone = context.clone();
                main_node = Some(parser_awaiting_value(&mut clone, current_token, &mut queue, target_data_type.clone(), descriptor.clone())?);
                context = clone;
            },
            ParsingContext::READY => todo!(),
            ParsingContext::AWAITING_FUNCTION(caller_node) => todo!(),
            ParsingContext::AWAITING_PARAMETERS(ref mut parameter_queue) => {
                if parameter_queue.len() == 0 {
                    // In this case, we know we have finished the function.
                    // This is purely a state change and syntax verification,
                    // so the code can stay in the main function
                    match current_token.token_type {
                        TokenType::CloseGroup(GroupType::Paren) => {
                            // All good, move on
                            context = ParsingContext::READY;
                        }
                        _ => {
                            return Err(GenerativeProgramCompileError::SyntaxError(SyntaxErrorType::ExpectedCloseParenthesis));
                        }
                    }
                } else {
                    // Safe to unwrap because len is non-zero
                    let target_data_type = parameter_queue.pop_front().unwrap();
                    let mut clone = context.clone();
                    match &mut main_node {
                        Some(BuilderNode::CombinationNode(_, vec)) => {
                            vec.push(parser_awaiting_value(&mut clone, current_token, &mut queue, target_data_type, descriptor.clone())?);
                        }
                        _ => {
                            return Err(GenerativeProgramCompileError::FoundValueWhileNotMakingCombinationNode)
                        }
                    }
                    
                    context = clone;
                }
            },
            ParsingContext::AWAITING_FUNCTION_BRACKET(params) => match current_token.token_type {
                TokenType::OpenGroup(GroupType::Paren) => {
                    context = ParsingContext::AWAITING_PARAMETERS(params)
                },
                _ => return Err(GenerativeProgramCompileError::SyntaxError(SyntaxErrorType::ExpectedOpenParenthesis))
            },
        }
    }

    todo!()
}

fn parser_awaiting_value(
    context: &mut ParsingContext,
    current_token: Token,
    other_tokens: &mut VecDeque<Token>,
    target_data_type: DataTypeDescriptor,
    descriptor: Rc<TableDescriptor>,
) -> Result<BuilderNode, GenerativeProgramCompileError> {
    match current_token.token_type {
        TokenType::Keyword(word) => match word {
            Keyword::Foreach => {
                *context = ParsingContext::AWAITING_FUNCTION_BRACKET(VecDeque::from(vec![DataTypeDescriptor::TableColumnSpecifier]));
                Ok(BuilderNode::CombinationNode(super::node_builder::FunctionType::Foreach, Vec::new()))
            }
            Keyword::Filter => todo!(),
            Keyword::Save => todo!(),
            Keyword::Saved => todo!(),
            _ => {
                return Err(GenerativeProgramCompileError::SyntaxError(
                    SyntaxErrorType::InvalidKeywordDuringBlankStageParsing,
                ))
            }
        },
        TokenType::NumericLiteral | TokenType::Symbol => {
            Ok(BuilderNode::GenericLiteral(create_literal_node(current_token, other_tokens, descriptor)?))
        },
        _ => {
            return Err(GenerativeProgramCompileError::SyntaxError(
                SyntaxErrorType::InvalidTokenDuringBlankStageParsing,
            ))
        }
    }
}

/// Returns underspecified literal node based on the tokens. Assumes the
/// context is correct for a literal, will error if not.
fn create_literal_node(
    current_token: Token,
    other_tokens: &mut VecDeque<Token>,
    descriptor: Rc<TableDescriptor>,
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
            match create_table_column_specifier(current_token.clone(), &mut secondary_queue, descriptor.clone()) {
                Ok(_) => {
                    // We need to apply the changes to the main
                    // queue. Unfortunately the easiest way to do
                    // this is to call the function again on the main
                    // queue.
                    // This operation should return successfully, as it
                    // did previous with clones of the parameters given.
                    let table_column =
                        create_table_column_specifier(current_token, other_tokens, descriptor.clone()).unwrap();

                    return Ok(UnderspecifiedLiteral::TableColumnSpecifier(table_column))
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
            let column_specifier = create_table_column_specifier(current_token, other_tokens, descriptor.clone())?;

            todo!()
        }
        _ => {
            return Err(GenerativeProgramCompileError::SyntaxError(
                SyntaxErrorType::InvalidTokenDuringKeywordParsing,
            ))
        }
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
    descriptor: Rc<TableDescriptor>,
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
                    // Specifying column by name
                    TokenType::Symbol => {
                        todo!()
                    }
                    _ => Err(GenerativeProgramCompileError::SyntaxError(
                        SyntaxErrorType::InvalidTokenDuringTableColumnSpecifierParsing(line!()),
                    )),
                }
            } else {
                Err(GenerativeProgramCompileError::SyntaxError(
                    SyntaxErrorType::InvalidTokenDuringTableColumnSpecifierParsing(line!()),
                ))
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
                                    // Specifying column by name
                                    TokenType::Symbol => {
                                        Ok(TableColumnSpecifier::BOTH(
                                            TableSpecifier { table_id },
                                            ColumnSpecifier { column_id: column_id_from_symbol(&next_token.token_contents, descriptor)? },
                                        ))
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
                        _ => Err(GenerativeProgramCompileError::SyntaxError(
                            SyntaxErrorType::InvalidTokenDuringTableColumnSpecifierParsing(line!()),
                        )),
                    }
                } else {
                    Err(GenerativeProgramCompileError::SyntaxError(
                        SyntaxErrorType::InvalidTokenDuringTableColumnSpecifierParsing(line!()),
                    ))
                }
            }
            Err(error) => Err(GenerativeProgramCompileError::IntParseError(error)),
        },
        _ => Err(GenerativeProgramCompileError::SyntaxError(
            SyntaxErrorType::InvalidTokenDuringTableColumnSpecifierParsing(line!()),
        )),
    }
}

/// This functions turns a symbol which should contain a column id into a 
/// column id to put into a descriptor
fn column_id_from_symbol(symbol: &str, descriptor: Rc<TableDescriptor>) -> Result<usize, GenerativeProgramCompileError> {
    for (i, column) in descriptor.column_descriptors.iter().enumerate() {
        if column.name == symbol {
            return Ok(i)
        }
    }
    
    return Err(GenerativeProgramCompileError::ColumnNotFound)
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
