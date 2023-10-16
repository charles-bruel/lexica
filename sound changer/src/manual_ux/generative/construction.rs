// Unify function parameter running

use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use crate::{
    main,
    manual_ux::{
        generative::{
            data_types::{Keyword, Operator},
            execution::{ComplexComparisionType, SimpleComparisionType},
            tokenizer::{GroupType, TokenType},
            SyntaxErrorType,
        },
        table::{
            GenerativeTableRowProcedure, TableDataTypeDescriptor, TableDescriptor,
            TableLoadingError, TableRow,
        },
    },
};

use super::{
    execution::{
        ColumnSpecifier, EnumNode, FilterPredicate, IntNode, RangeNode, StringNode, TableSpecifier,
        UIntNode,
    },
    node_builder::{BuilderNode, FunctionType, UnderspecifiedLiteral},
    tokenizer::{tokenize, Token},
    GenerativeProgram, GenerativeProgramCompileError,
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum DataTypeDescriptor {
    TableDataType(TableDataTypeDescriptor),
    TableColumnSpecifier,
    FilterPredicate,
    Range,
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
    /// call or member should follow.
    /// The parameter contains the type of the object this was called on
    AWAITING_ACCESS,
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
    /// Indicates that we have recieved a function and we have at least 1
    /// left.
    /// TODO: Find a way to allow trailing commas (i.e. `(a, b, c,)`)
    AWAITING_FUNCTION_COMMA(VecDeque<DataTypeDescriptor>),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub enum TableColumnSpecifier {
    TABLE(TableSpecifier),
    COLUMN(ColumnSpecifier),
    BOTH(TableSpecifier, ColumnSpecifier),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct EnumSpecifier {
    pub name: String,
    pub column: ColumnSpecifier,
    pub table: Option<TableSpecifier>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ProjectContext {
    pub descriptor: Rc<TableDescriptor>,
    pub all_descriptors: HashMap<usize, Rc<TableDescriptor>>,
    pub table_id: usize,
    pub current_column_id: usize,
}

pub fn parse_generative_table_line(
    all_descriptors: HashMap<usize, Rc<TableDescriptor>>,
    table_id: usize,
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
            all_descriptors.clone(),
            table_id,
        ))?),
        descriptor: all_descriptors[&table_id].clone(),
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
    all_descriptors: HashMap<usize, Rc<TableDescriptor>>,
    table_id: usize,
) -> Result<GenerativeTableRowProcedure, GenerativeProgramCompileError> {
    let mut context: ProjectContext = ProjectContext {
        descriptor: all_descriptors[&table_id].clone(),
        all_descriptors,
        table_id,
        current_column_id: 0,
    };

    let mut result: Vec<GenerativeProgram> = Vec::with_capacity(token_sets.len());
    let mut index = 0;
    for tokens in token_sets {
        result.push(create_generative_program(
            tokens,
            &context.clone().descriptor.column_descriptors[index].data_type,
            &mut context,
        )?);
        context.current_column_id += 1;
        index += 1;
    }
    todo!()
}

fn create_generative_program(
    tokens: Vec<Token>,
    output_type: &TableDataTypeDescriptor,
    project_context: &mut ProjectContext,
) -> Result<GenerativeProgram, GenerativeProgramCompileError> {
    let main_segment = parse_generative_segment(true, tokens, output_type, project_context)?;

    Ok(GenerativeProgram {
        output_node: main_segment.try_convert_output_node(&output_type, project_context)?,
    })
}

fn parse_generative_segment(
    require_equals: bool,
    tokens: Vec<Token>,
    output_type: &TableDataTypeDescriptor,
    project_context: &mut ProjectContext,
) -> Result<BuilderNode, GenerativeProgramCompileError> {
    let mut context: VecDeque<ParsingContext> = if require_equals {
        VecDeque::from(vec![ParsingContext::START])
    } else {
        VecDeque::from(vec![ParsingContext::AWAITING_VALUE(
            DataTypeDescriptor::TableDataType(output_type.clone()),
        )])
    };

    let mut queue = VecDeque::from(tokens);

    let mut main_node: Option<BuilderNode> = None;

    while queue.len() > 0 {
        // Queue is garunteed to have elements because of while condition
        let current_token = queue.pop_front().unwrap();

        println!("{:?}:\n\t{:?}\n\t{:?}", context, current_token, main_node);

        match &mut context[0] {
            // TODO: Split into own function
            ParsingContext::START => match current_token.token_type {
                TokenType::Operator(Operator::Equals) => {
                    context[0] = ParsingContext::AWAITING_VALUE(DataTypeDescriptor::TableDataType(
                        output_type.clone(),
                    ))
                }
                _ => {
                    return Ok(BuilderNode::GenericLiteral(create_literal_node(
                        current_token,
                        &mut queue,
                        project_context,
                    )?))
                }
            },
            ParsingContext::AWAITING_VALUE(ref target_data_type) => {
                let (node, optional_addition) = parse_awaiting_value(
                    current_token,
                    &mut queue,
                    &target_data_type,
                    project_context,
                )?;
                main_node = Some(node);
                context[0] = ParsingContext::READY;
                if let Some(v) = optional_addition {
                    context.push_front(v);
                }
            }
            ParsingContext::READY => {
                let mut top_context = context[0].clone();
                main_node = Some(parse_new_segment_ready(
                    &mut top_context,
                    current_token,
                    main_node,
                )?);
                context[0] = top_context;
            }
            ParsingContext::AWAITING_ACCESS => match &mut main_node {
                Some(BuilderNode::CombinationNode(function_type, _)) => {
                    let (returned_function_type, new_context) = parse_access(current_token)?;
                    *function_type = returned_function_type;
                    context[0] = ParsingContext::READY;
                    if let Some(v) = new_context {
                        context.push_front(v);
                    }
                }
                _ => todo!(),
            },
            ParsingContext::AWAITING_PARAMETERS(ref mut parameter_queue) => {
                if parameter_queue.len() == 0 {
                    // In this case, we know we have finished the function.
                    // This is purely a state change and syntax verification,
                    // so the code can stay in the main function
                    match current_token.token_type {
                        TokenType::CloseGroup(GroupType::Paren) => {
                            // All good, move on
                            // All entries to the function system *should* add a new layer
                            // to the context stack, however this is not 100% safe to assume
                            // and should be checked.
                            context.pop_front();
                            if context.len() == 0 {
                                return Err(GenerativeProgramCompileError::SyntaxError(
                                    SyntaxErrorType::UnbalancedFunctions,
                                ));
                            }
                        }
                        _ => {
                            return Err(GenerativeProgramCompileError::SyntaxError(
                                SyntaxErrorType::ExpectedCloseParenthesis,
                            ));
                        }
                    }
                } else {
                    // Safe to unwrap because len is non-zero
                    let target_data_type = parameter_queue.pop_front().unwrap();
                    let cloned_queue = parameter_queue.clone();
                    match &mut main_node {
                        Some(BuilderNode::CombinationNode(_, vec)) => {
                            let (node, parsing_context) = parse_awaiting_value(
                                current_token,
                                &mut queue,
                                &target_data_type,
                                project_context,
                            )?;
                            if cloned_queue.len() != 0 {
                                context[0] = ParsingContext::AWAITING_FUNCTION_COMMA(cloned_queue);
                            }
                            vec.push(node);
                            if let Some(v) = parsing_context {
                                context.push_front(v)
                            }
                        }
                        _ => return Err(
                            GenerativeProgramCompileError::FoundValueWhileNotMakingCombinationNode,
                        ),
                    }
                }
            }
            ParsingContext::AWAITING_FUNCTION_BRACKET(params) => match current_token.token_type {
                TokenType::OpenGroup(GroupType::Paren) => {
                    context[0] = ParsingContext::AWAITING_PARAMETERS(params.clone())
                }
                _ => {
                    return Err(GenerativeProgramCompileError::SyntaxError(
                        SyntaxErrorType::ExpectedOpenParenthesis,
                    ))
                }
            },
            ParsingContext::AWAITING_FUNCTION_COMMA(_) => todo!(),
        }
    }

    match main_node {
        Some(v) => Ok(v),
        None => Err(GenerativeProgramCompileError::NoValueFromSegment),
    }
}

/// This function handles the ready state of parsing.
/// There are two ways out of the ready state - a combination
/// symbol (`+` or sometimes `-`) or a function starter (.)
fn parse_new_segment_ready(
    context: &mut ParsingContext,
    current_token: Token,
    main_node: Option<BuilderNode>,
) -> Result<BuilderNode, GenerativeProgramCompileError> {
    match current_token.token_type {
        TokenType::Operator(Operator::Plus | Operator::Minus) => {
            todo!()
        }
        TokenType::Operator(Operator::Period) => {
            // We have a function and the first parameter
            // will be the current main node
            *context = ParsingContext::AWAITING_ACCESS;
            Ok(BuilderNode::CombinationNode(
                super::node_builder::FunctionType::UnknownFunction,
                vec![match main_node {
                    Some(v) => v,
                    None => return Err(GenerativeProgramCompileError::MainNodeHasNoValue),
                }],
            ))
        }
        _ => {
            todo!()
        }
    }
}

fn parse_access(
    current_token: Token,
) -> Result<(FunctionType, Option<ParsingContext>), GenerativeProgramCompileError> {
    match current_token.token_type {
        TokenType::Keyword(word) => match word {
            Keyword::Foreach | Keyword::Saved => Err(GenerativeProgramCompileError::SyntaxError(
                SyntaxErrorType::FunctionForbidsObject,
            )),
            Keyword::Filter => Ok((
                FunctionType::Filter,
                Some(ParsingContext::AWAITING_FUNCTION_BRACKET(VecDeque::from(
                    vec![DataTypeDescriptor::FilterPredicate],
                ))),
            )),
            Keyword::Save => Ok((
                FunctionType::Save,
                Some(ParsingContext::AWAITING_FUNCTION_BRACKET(VecDeque::from(
                    vec![DataTypeDescriptor::TableDataType(
                        TableDataTypeDescriptor::String,
                    )],
                ))),
            )),
            _ => {
                return Err(GenerativeProgramCompileError::SyntaxError(
                    SyntaxErrorType::InvalidKeywordDuringBlankStageParsing,
                ))
            }
        },
        TokenType::Symbol => Ok((
            FunctionType::SymbolLookup(current_token.token_contents),
            None,
        )),
        _ => {
            return Err(GenerativeProgramCompileError::SyntaxError(
                SyntaxErrorType::InvalidTokenDuringBlankStageParsing,
            ))
        }
    }
}

fn parse_filter_predicate_expression(
    mut current_token: Token,
    other_tokens: &mut VecDeque<Token>,
    project_context: &mut ProjectContext,
) -> Result<FilterPredicate, GenerativeProgramCompileError> {
    let mut lhs: Vec<Token> = Vec::new();
    let mut rhs: Vec<Token> = Vec::new();

    // Find LHS and comparision type
    let comparision_type = loop {
        // TODO: Also end on `)` and `,`
        // TODO: Another check that tracks nesting depth, to protect `)`, `,`, and comparision operators within parenthesis
        match current_token.token_type {
            TokenType::Operator(operator) => {
                match operator {
                    Operator::Equality
                    | Operator::Inequality
                    | Operator::Greater
                    | Operator::GreaterEqual
                    | Operator::Less
                    | Operator::LessEqual => {
                        // We've found the comparator in the expression. Drop the token, grab the next one, and move on.
                        current_token = match other_tokens.pop_front() {
                            Some(v) => v,
                            None => {
                                return Err(GenerativeProgramCompileError::SyntaxError(
                                    SyntaxErrorType::FilterPredicateEndsEarly,
                                ))
                            }
                        };
                        break operator;
                    }
                    _ => {
                        lhs.push(current_token);
                        current_token = match other_tokens.pop_front() {
                            Some(v) => v,
                            None => {
                                return Err(GenerativeProgramCompileError::SyntaxError(
                                    SyntaxErrorType::FilterPredicateEndsEarly,
                                ))
                            }
                        };
                    }
                }
            }
            _ => {
                lhs.push(current_token);
                current_token = match other_tokens.pop_front() {
                    Some(v) => v,
                    None => {
                        return Err(GenerativeProgramCompileError::SyntaxError(
                            SyntaxErrorType::FilterPredicateEndsEarly,
                        ))
                    }
                };
            }
        }
    };

    // Find RHS
    loop {
        // TODO: Another check that tracks nesting depth, to protect `)` and `,` operators within parenthesis
        match current_token.token_type {
            TokenType::Operator(Operator::Comma) | TokenType::CloseGroup(GroupType::Paren) => {
                // We've found the end of the expression. We need to realign the queue
                other_tokens.push_front(current_token.clone());
                break;
            }
            _ => {
                rhs.push(current_token);
                current_token = match other_tokens.pop_front() {
                    Some(v) => v,
                    None => {
                        return Err(GenerativeProgramCompileError::SyntaxError(
                            SyntaxErrorType::FilterPredicateEndsEarly,
                        ))
                    }
                };
            }
        }
    }

    // We need to determine which of the RHS or LHS is the column specifier
    let lhs_column_specifier = check_if_column_specifier(lhs.clone(), project_context);
    let rhs_column_specifier = check_if_column_specifier(rhs.clone(), project_context);

    if lhs_column_specifier == rhs_column_specifier {
        // Only one can be the specifier
        // TODO: Allow both to be specifiers to check for column equality in a row
        // Also TODO: Actual error handling here
        todo!()
    }

    let (specifier, predicate) = if lhs_column_specifier {
        (lhs, rhs)
    } else {
        (rhs, lhs)
    };

    let table_column_specifier = {
        let mut other = VecDeque::from(specifier);
        // If there weren't true, the check_if_column_specifier would have
        // returned false and we wouldn't be here.
        let current_token = other.pop_front().unwrap();
        create_table_column_specifier(current_token, &mut other, project_context)?
    };

    let column = match table_column_specifier {
        TableColumnSpecifier::COLUMN(v) => v,
        _ => return Err(GenerativeProgramCompileError::FilterPredicateSpecifierColumnOnly),
    };

    let output_type =
        &project_context.clone().descriptor.column_descriptors[column.column_id].data_type;

    let predicate_segment =
        parse_generative_segment(false, predicate, &output_type.clone(), project_context)?;

    match output_type {
        TableDataTypeDescriptor::Enum(enum_values) => {
            let final_predicate = predicate_segment.try_convert_enum(project_context)?;
            let simple_comparision_type = match comparision_type {
                Operator::Equality => SimpleComparisionType::Equals,
                Operator::Inequality => SimpleComparisionType::NotEquals,
                _ => return Err(GenerativeProgramCompileError::OnlyEqualsAndNotEqualsValidHere),
            };
            return Ok(FilterPredicate::EnumCompare(
                column,
                simple_comparision_type,
                final_predicate,
            ));
        }
        TableDataTypeDescriptor::String => {
            let final_predicate = predicate_segment.try_convert_string(project_context)?;
            let simple_comparision_type = match comparision_type {
                Operator::Equality => SimpleComparisionType::Equals,
                Operator::Inequality => SimpleComparisionType::NotEquals,
                _ => return Err(GenerativeProgramCompileError::OnlyEqualsAndNotEqualsValidHere),
            };
            return Ok(FilterPredicate::StringCompare(
                column,
                simple_comparision_type,
                final_predicate,
            ));
        }
        TableDataTypeDescriptor::UInt => {
            let final_predicate = predicate_segment.try_convert_uint(project_context)?;
            let complex_comparision_type = match comparision_type {
                Operator::Equality => ComplexComparisionType::Equals,
                Operator::Inequality => ComplexComparisionType::NotEquals,
                Operator::Greater => ComplexComparisionType::Greater,
                Operator::GreaterEqual => ComplexComparisionType::GreaterEquals,
                Operator::Less => ComplexComparisionType::Less,
                Operator::LessEqual => ComplexComparisionType::LessEquals,
                // Since to pull it as an comparision type, it needs to be one of
                // the above values, this will be unreachable
                _ => unreachable!(),
            };
            return Ok(FilterPredicate::UIntCompare(
                column,
                complex_comparision_type,
                final_predicate,
            ));
        }
        TableDataTypeDescriptor::Int => {
            let final_predicate = predicate_segment.try_convert_int(project_context)?;
            let complex_comparision_type = match comparision_type {
                Operator::Equality => ComplexComparisionType::Equals,
                Operator::Inequality => ComplexComparisionType::NotEquals,
                Operator::Greater => ComplexComparisionType::Greater,
                Operator::GreaterEqual => ComplexComparisionType::GreaterEquals,
                Operator::Less => ComplexComparisionType::Less,
                Operator::LessEqual => ComplexComparisionType::LessEquals,
                // Since to pull it as an comparision type, it needs to be one of
                // the above values, this will be unreachable
                _ => unreachable!(),
            };
            return Ok(FilterPredicate::IntCompare(
                column,
                complex_comparision_type,
                final_predicate,
            ));
        }
    }
}

/// This function is used within parse_filter_predicate_expression() to determine
/// if a given side of the equation is the column specifier.
fn check_if_column_specifier(tokens: Vec<Token>, project_context: &mut ProjectContext) -> bool {
    let mut deque = VecDeque::from(tokens);
    let first = match deque.pop_front() {
        Some(v) => v,
        None => return false, // Not our problem
    };
    let result = create_table_column_specifier(first, &mut deque, project_context);

    if deque.len() != 0 {
        // There are tokens left, meaning this expression
        // is not purely a column specifier.
        return false;
    }

    match result {
        Ok(_) => return true,
        Err(_) => return false,
    }
}

/// Begins parses any value expressions, whether that be a function
/// or a literal. May begin function processing with an optional parsing
/// context, for use within an inner segment.
fn parse_awaiting_value(
    current_token: Token,
    other_tokens: &mut VecDeque<Token>,
    target_data_type: &DataTypeDescriptor,
    project_context: &mut ProjectContext,
) -> Result<(BuilderNode, Option<ParsingContext>), GenerativeProgramCompileError> {
    match target_data_type {
        DataTypeDescriptor::FilterPredicate => {
            // We have to return a filter predicate for the program to be valid,
            // so unlike the case with the table column specifier literal, we can
            // pull elements from the token queue without any checks or backups.
            return Ok((
                BuilderNode::FilterPredicate(parse_filter_predicate_expression(
                    current_token,
                    other_tokens,
                    project_context,
                )?),
                None,
            ));
        }
        _ => {}
    }
    match current_token.token_type {
        TokenType::Keyword(word) => match word {
            Keyword::Foreach => Ok((
                BuilderNode::CombinationNode(
                    super::node_builder::FunctionType::Foreach,
                    Vec::new(),
                ),
                Some(ParsingContext::AWAITING_FUNCTION_BRACKET(VecDeque::from(
                    vec![DataTypeDescriptor::TableColumnSpecifier],
                ))),
            )),
            Keyword::Filter | Keyword::Save => Err(GenerativeProgramCompileError::SyntaxError(
                SyntaxErrorType::FunctionRequiresObject,
            )),
            Keyword::Saved => todo!(),
            _ => Err(GenerativeProgramCompileError::SyntaxError(
                SyntaxErrorType::InvalidKeywordDuringBlankStageParsing,
            )),
        },
        TokenType::NumericLiteral | TokenType::Symbol => Ok((
            BuilderNode::GenericLiteral(create_literal_node(
                current_token,
                other_tokens,
                project_context,
            )?),
            None,
        )),
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
    project_context: &mut ProjectContext,
) -> Result<UnderspecifiedLiteral, GenerativeProgramCompileError> {
    match current_token.token_type {
        // String or enum
        TokenType::Symbol => Ok(UnderspecifiedLiteral::StringOrShortEnum(
            current_token.token_contents,
            ColumnSpecifier {
                column_id: project_context.current_column_id,
            },
            TableSpecifier {
                table_id: project_context.table_id,
            },
        )),
        // Int or uint or enum
        TokenType::NumericLiteral => {
            // First we check if it's an enum definition with a
            // table-column specifier out the front. We will attempt
            // to form a table-column specifier to do this. Since
            // that operation affects the VecDeque of tokens, we
            // need to clone it first, and only if the operation
            // suceeds do we update the main variable.
            let mut secondary_queue = other_tokens.clone();
            match create_table_column_specifier(
                current_token.clone(),
                &mut secondary_queue,
                project_context,
            ) {
                Ok(_) => {
                    // We need to apply the changes to the main
                    // queue. Unfortunately the easiest way to do
                    // this is to call the function again on the main
                    // queue.
                    // This operation should return successfully, as it
                    // did previous with clones of the parameters given.
                    let table_column =
                        create_table_column_specifier(current_token, other_tokens, project_context)
                            .unwrap();

                    return Ok(UnderspecifiedLiteral::TableColumnSpecifier(table_column));
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
            let column_specifier =
                create_table_column_specifier(current_token, other_tokens, project_context)?;

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
    project_context: &mut ProjectContext,
) -> Result<TableColumnSpecifier, GenerativeProgramCompileError> {
    // This function is only the finest of too many levels of nesting
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
                    TokenType::Symbol => Ok(TableColumnSpecifier::COLUMN(ColumnSpecifier {
                        column_id: column_id_from_symbol(
                            &next_token.token_contents,
                            project_context,
                        )?,
                    })),
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
                                    TokenType::Symbol => Ok(TableColumnSpecifier::BOTH(
                                        TableSpecifier { table_id },
                                        ColumnSpecifier {
                                            column_id: column_id_from_symbol(
                                                &next_token.token_contents,
                                                project_context,
                                            )?,
                                        },
                                    )),
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
fn column_id_from_symbol(
    symbol: &str,
    project_context: &mut ProjectContext,
) -> Result<usize, GenerativeProgramCompileError> {
    for (i, column) in project_context
        .descriptor
        .column_descriptors
        .iter()
        .enumerate()
    {
        if column.name == symbol {
            return Ok(i);
        }
    }

    return Err(GenerativeProgramCompileError::ColumnNotFound);
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
