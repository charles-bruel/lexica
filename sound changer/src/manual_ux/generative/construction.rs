// Unify function parameter running

use std::{
    collections::{HashMap, VecDeque},
    mem,
    rc::Rc,
};

use crate::manual_ux::{
    generative::{
        compile_err, compile_err_token,
        data_types::{Keyword, Operator},
        execution::{ComplexComparisionType, SimpleComparisionType},
        tokenizer::{GroupType, TokenType},
        SyntaxErrorType,
    },
    table::{
        loading_err, GenerativeTableRowProcedure, LoadingErrorType, TableDataTypeDescriptor,
        TableDescriptor, TableLoadingError, TableRow,
    },
};

use super::{
    execution::{ColumnSpecifier, FilterPredicate, TableSpecifier},
    node_builder::{BuilderNode, FinalOperandIndex, FunctionType, UnderspecifiedLiteral},
    tokenizer::{tokenize, Token},
    CompileErrorType, GenerativeProgram, GenerativeProgramCompileError,
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
    Start,
    /// Indicates there is no relevant context in the parser and that
    /// it should receive a value of the given type.
    /// Occurs after the start state, and after an operator such as `+`
    /// (not `.`). The code associated with this state is also called from
    /// the `AwaitingParameters` state.
    AwaitingValue(Option<DataTypeDescriptor>),
    /// Indicates that there is a valid construction of the given
    /// data type and the operation can be finished or expanded with
    /// any number of operators or symbols, such as `.` or ` + `
    Ready,
    /// Indicates that a `.` symbol was correctly used so a function
    /// call or member should follow.
    /// The parameter contains the type of the object this was called on
    AwaitingAccess,
    /// Indicates that function keyword was used and we now need the
    /// opening `(` of the function.
    /// The value stored is the eventual parameters of the function
    /// (it will be determined when this is set, so it is transfer
    /// through here to it's final destination).
    AwaitingFunctionBracket(VecDeque<DataTypeDescriptor>),
    /// Indicates that we are in a function and awaiting some number of
    /// parameters, or that the function has concluded and we are awaiting
    /// the closing `)`.
    /// The value it stores is the types of the upcoming parameters;
    /// it is also how the system keeps track of the number of parameters
    /// left.
    AwaitingParameters(VecDeque<DataTypeDescriptor>),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub enum TableColumnSpecifier {
    Table(TableSpecifier),
    Column(ColumnSpecifier),
    Both(TableSpecifier, ColumnSpecifier),
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
    if line.len() < 3 {
        return loading_err(LoadingErrorType::GenerativeProgramCompileError(
            GenerativeProgramCompileError {
                error_type: CompileErrorType::SyntaxError(
                    SyntaxErrorType::MissingProgramSurrondings,
                ),
                attribution: super::CompileAttribution::None,
            },
        ));
    }

    if &line[0..3] != ":={" || line.chars().nth_back(0).unwrap() != '}' {
        return loading_err(LoadingErrorType::GenerativeProgramCompileError(
            GenerativeProgramCompileError {
                error_type: CompileErrorType::SyntaxError(
                    SyntaxErrorType::MissingProgramSurrondings,
                ),
                attribution: super::CompileAttribution::None,
            },
        ));
    }

    if line.len() <= 4 {
        return loading_err(LoadingErrorType::GenerativeProgramCompileError(
            GenerativeProgramCompileError {
                error_type: CompileErrorType::SyntaxError(SyntaxErrorType::NoGenerativeContent),
                attribution: super::CompileAttribution::None,
            },
        ));
    }

    let tokens = tokenize(line[3..line.len() - 2].to_string());

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

    Ok(TableRow::UnpopulatedTableRow {
        procedure: Rc::new(convert_error(create_generative_table_row_procedure(
            token_sets,
            all_descriptors.clone(),
            table_id,
        ))?),
        descriptor: all_descriptors[&table_id].clone(),
    })
}

fn convert_error<T>(
    result: Result<T, GenerativeProgramCompileError>,
) -> Result<T, TableLoadingError> {
    match result {
        Ok(v) => Ok(v),
        Err(e) => loading_err(LoadingErrorType::GenerativeProgramCompileError(e)),
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
    for (index, tokens) in token_sets.into_iter().enumerate() {
        result.push(create_generative_program(
            tokens,
            &context.clone().descriptor.column_descriptors[index].data_type,
            &mut context,
        )?);
        context.current_column_id += 1;
    }

    Ok(GenerativeTableRowProcedure { programs: result })
}

fn create_generative_program(
    tokens: Vec<Token>,
    output_type: &TableDataTypeDescriptor,
    project_context: &mut ProjectContext,
) -> Result<GenerativeProgram, GenerativeProgramCompileError> {
    let queue = &mut VecDeque::from(tokens);
    let main_segment = parse_generative_segment(
        true,
        queue,
        &Some(output_type.clone()),
        project_context,
        None,
    )?;

    Ok(GenerativeProgram {
        output_node: main_segment.try_convert_output_node(output_type, project_context)?,
    })
}

// TODO: Allow any data type in some circumstances
fn parse_generative_segment(
    require_equals: bool,
    tokens: &mut VecDeque<Token>,
    output_type: &Option<TableDataTypeDescriptor>,
    project_context: &mut ProjectContext,
    closing_token_type: Option<TokenType>,
) -> Result<BuilderNode, GenerativeProgramCompileError> {
    let mut context: VecDeque<ParsingContext> = if require_equals {
        VecDeque::from(vec![ParsingContext::Start])
    } else {
        VecDeque::from(vec![ParsingContext::AwaitingValue(
            output_type
                .as_ref()
                .map(|v| DataTypeDescriptor::TableDataType(v.clone())),
        )])
    };

    // let mut tokens: VecDeque<Token> = VecDeque::from(tokens);

    let mut main_node: Option<BuilderNode> = None;

    while !tokens.is_empty() {
        // Queue is garunteed to have elements because of while condition
        let current_token = tokens.pop_front().unwrap();

        match &mut context[0] {
            // TODO: Split into own function
            ParsingContext::Start => match current_token.token_type {
                TokenType::Operator(Operator::Equals) => {
                    context[0] = ParsingContext::AwaitingValue(
                        output_type
                            .as_ref()
                            .map(|v| DataTypeDescriptor::TableDataType(v.clone())),
                    )
                }
                _ => {
                    return Ok(BuilderNode::GenericLiteral(create_literal_node(
                        current_token,
                        tokens,
                        project_context,
                    )?))
                }
            },
            ParsingContext::AwaitingValue(ref target_data_type) => {
                let (node, optional_addition) = parse_awaiting_value(
                    current_token,
                    tokens,
                    target_data_type,
                    project_context,
                    None,
                )?;
                match &mut main_node {
                    Some(BuilderNode::CombinationNode(_, v, _)) => v.push(node),
                    None => main_node = Some(node),
                    _ => {}
                }

                context[0] = ParsingContext::Ready;
                if let Some(v) = optional_addition {
                    context.push_front(v);
                }
            }
            ParsingContext::Ready => {
                if let Some(token_type) = closing_token_type {
                    if token_type == current_token.token_type {
                        return match main_node {
                            Some(v) => Ok(BuilderNode::Wrapper(Box::new(v))),
                            None => compile_err_token(
                                CompileErrorType::NoValueFromSegment,
                                current_token,
                            ),
                        };
                    }
                }

                let mut top_context = context[0].clone();
                main_node = Some(parse_new_segment_ready(
                    &mut top_context,
                    current_token,
                    main_node,
                )?);
                context[0] = top_context;
            }
            ParsingContext::AwaitingAccess => match &mut main_node {
                Some(v) => match v.get_function_node() {
                    BuilderNode::CombinationNode(function_type, _, _) => {
                        let (returned_function_type, new_context) = parse_access(current_token)?;
                        *function_type = returned_function_type;
                        context[0] = ParsingContext::Ready;
                        if let Some(v) = new_context {
                            context.push_front(v);
                        }
                    }
                    _ => todo!(),
                },
                _ => panic!(),
            },
            ParsingContext::AwaitingParameters(ref mut parameter_queue) => {
                if parameter_queue.is_empty() {
                } else {
                    // Actually fill in function parameters
                    // Safe to unwrap because len is non-zero
                    let target_data_type = parameter_queue.pop_front().unwrap();

                    // TODO: Find way to allow trailing commas
                    let automatic_recursion_end_token = Some(if parameter_queue.is_empty() {
                        TokenType::CloseGroup(GroupType::Paren)
                    } else {
                        TokenType::Operator(Operator::Comma)
                    });
                    let (node, parsing_context) = parse_awaiting_value(
                        current_token,
                        tokens,
                        &Some(target_data_type),
                        project_context,
                        automatic_recursion_end_token,
                    )?;
                    if parameter_queue.is_empty() {
                        context.pop_front();
                        if context.is_empty() {
                            if tokens.is_empty() {
                                return compile_err(CompileErrorType::SyntaxError(
                                    SyntaxErrorType::UnbalancedFunctions,
                                ));
                            } else {
                                return compile_err_token(
                                    CompileErrorType::SyntaxError(
                                        SyntaxErrorType::UnbalancedFunctions,
                                    ),
                                    tokens.pop_front().unwrap(),
                                );
                            }
                        }
                    }
                    match &mut main_node {
                        Some(v) => v.insert_operand(node),
                        None => panic!(),
                    }
                    if let Some(v) = parsing_context {
                        context.push_front(v)
                    }
                }
            }

            // Syntax guarantees that don't affect the structure
            ParsingContext::AwaitingFunctionBracket(params) => match current_token.token_type {
                TokenType::OpenGroup(GroupType::Paren) => {
                    context[0] = ParsingContext::AwaitingParameters(params.clone())
                }
                _ => {
                    return compile_err_token(
                        CompileErrorType::SyntaxError(SyntaxErrorType::ExpectedOpenParenthesis),
                        current_token,
                    )
                }
            },
        }
    }

    // TODO: Check that we are in a good parsing state before returning
    match main_node {
        Some(v) => Ok(BuilderNode::Wrapper(Box::new(v))),
        None => compile_err(CompileErrorType::NoValueFromSegment),
    }
}

/// This function handles the ready state of parsing.
/// There are two ways out of the ready state - a combination
/// symbol (`+` or sometimes `-`) or a function starter (`.`)
fn parse_new_segment_ready(
    context: &mut ParsingContext,
    current_token: Token,
    main_node: Option<BuilderNode>,
) -> Result<BuilderNode, GenerativeProgramCompileError> {
    let old_node = match main_node {
        Some(v) => v,
        None => return compile_err_token(CompileErrorType::MainNodeHasNoValue, current_token),
    };

    match &current_token.token_type {
        // Low precedence
        TokenType::Operator(v @ (Operator::Plus | Operator::Minus)) => {
            *context = ParsingContext::AwaitingValue(None);

            let function_type = if v == &Operator::Plus {
                FunctionType::Addition
            } else {
                FunctionType::Subtraction
            };

            Ok(insert_new_operator(old_node, function_type, 0)?)
        }
        // Medium precedence
        TokenType::Operator(Operator::Star) => todo!(),
        // High precendence
        TokenType::Operator(Operator::Period) => {
            // We have a function and the first parameter
            // will be the current main node
            *context = ParsingContext::AwaitingAccess;

            Ok(insert_new_operator(
                old_node,
                FunctionType::UnknownFunction,
                2,
            )?)
        }
        _ => {
            panic!()
        }
    }
}

fn insert_new_operator(
    mut old_node: BuilderNode,
    function_type: FunctionType,
    precedence: u8,
) -> Result<BuilderNode, GenerativeProgramCompileError> {
    if let BuilderNode::CombinationNode(old_function_type, vec, old_precedence) = &mut old_node {
        if precedence > *old_precedence {
            // In this case, we have that the new precedence is greater
            // so we need to replace the last operand of the previous node
            // with a combination of itself and this.

            // i.e. `a + b => a + b.c() <=> a + c(b)`

            // TODO: DRY
            return match old_function_type.final_operand_index() {
                FinalOperandIndex::First => {
                    let old_vec = vec;
                    let new_parameter_vec = vec![];
                    // `a + b => a + c()`
                    let idx = 0;
                    let old_operand = mem::replace(
                        &mut old_vec[idx],
                        BuilderNode::CombinationNode(function_type, new_parameter_vec, precedence),
                    );
                    // `a + c() => a + c(b) <=> a + b.c()`
                    if let BuilderNode::CombinationNode(_, vec, _) = &mut old_vec[idx] {
                        vec.push(old_operand);
                    } else {
                        // We just assigned old_vec[idx] to be a BuilderNode::CombinationNode
                        // and then immediately execute this, so this is guaranteed.
                        unreachable!()
                    }
                    Ok(old_node)
                }
                FinalOperandIndex::Last => {
                    let old_vec = vec;
                    let new_parameter_vec = vec![];
                    // `a + b => a + c()`
                    let idx = old_vec.len() - 1;
                    let old_operand = mem::replace(
                        &mut old_vec[idx],
                        BuilderNode::CombinationNode(function_type, new_parameter_vec, precedence),
                    );
                    // `a + c() => a + c(b) <=> a + b.c()`
                    if let BuilderNode::CombinationNode(_, vec, _) = &mut old_vec[idx] {
                        vec.push(old_operand);
                    } else {
                        // We just assigned old_vec[idx] to be a BuilderNode::CombinationNode
                        // and then immediately execute this, so this is guaranteed.
                        unreachable!()
                    }
                    Ok(old_node)
                }
            };
        }

        // The old operation has greater or equal precedence
        // (i.e. 5 * 5 + 3 or 5 + 5 + 3) We want to operate on
        // the old operation as a block so we fall through and let
        // the default handling work.
    }

    // Old node is not a combination node; there is no way for precedence to be weird here, so we use default behavior.
    Ok(BuilderNode::CombinationNode(
        function_type,
        vec![old_node],
        precedence,
    ))
}

fn parse_access(
    current_token: Token,
) -> Result<(FunctionType, Option<ParsingContext>), GenerativeProgramCompileError> {
    match current_token.token_type {
        TokenType::Keyword(word) => match word {
            Keyword::Foreach | Keyword::Saved => compile_err_token(
                CompileErrorType::SyntaxError(SyntaxErrorType::FunctionForbidsObject),
                current_token,
            ),
            Keyword::Filter => Ok((
                FunctionType::Filter,
                Some(ParsingContext::AwaitingFunctionBracket(VecDeque::from(
                    vec![DataTypeDescriptor::FilterPredicate],
                ))),
            )),
            Keyword::Save => Ok((
                FunctionType::Save,
                Some(ParsingContext::AwaitingFunctionBracket(VecDeque::from(
                    vec![DataTypeDescriptor::TableDataType(
                        TableDataTypeDescriptor::String,
                    )],
                ))),
            )),
            Keyword::SoundChange => Ok((
                FunctionType::SoundChange,
                Some(ParsingContext::AwaitingFunctionBracket(VecDeque::from(
                    vec![DataTypeDescriptor::TableDataType(
                        TableDataTypeDescriptor::String,
                    )],
                ))),
            )),
            Keyword::Mutate => Ok((
                FunctionType::Mutate,
                Some(ParsingContext::AwaitingFunctionBracket(VecDeque::from(
                    vec![
                        DataTypeDescriptor::Range,
                        DataTypeDescriptor::TableDataType(TableDataTypeDescriptor::UInt),
                    ],
                ))),
            )),
            _ => compile_err_token(
                CompileErrorType::SyntaxError(
                    SyntaxErrorType::InvalidKeywordDuringBlankStageParsing,
                ),
                current_token,
            ),
        },
        TokenType::Symbol => Ok((
            FunctionType::SymbolLookup(current_token.token_contents),
            None,
        )),
        _ => compile_err_token(
            CompileErrorType::SyntaxError(SyntaxErrorType::InvalidTokenDuringBlankStageParsing),
            current_token,
        ),
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
                                return compile_err_token(
                                    CompileErrorType::SyntaxError(
                                        SyntaxErrorType::FilterPredicateEndsEarly,
                                    ),
                                    current_token,
                                )
                            }
                        };
                        break operator;
                    }
                    _ => {
                        lhs.push(current_token);
                        current_token = match other_tokens.pop_front() {
                            Some(v) => v,
                            None => {
                                return compile_err(CompileErrorType::SyntaxError(
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
                        return compile_err(CompileErrorType::SyntaxError(
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
                other_tokens.push_front(current_token);
                break;
            }
            _ => {
                rhs.push(current_token);
                current_token = match other_tokens.pop_front() {
                    Some(v) => v,
                    None => {
                        return compile_err(CompileErrorType::SyntaxError(
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

    let (table, column) = match table_column_specifier {
        TableColumnSpecifier::Column(v) => (project_context.table_id, v),
        TableColumnSpecifier::Both(t, v) => (t.table_id, v),
        _ => {
            return compile_err_token(
                CompileErrorType::FilterPredicateSpecifierRequireColumn,
                other_tokens.pop_front().unwrap(),
            )
        }
    };

    let output_type = &project_context.all_descriptors[&table].column_descriptors[column.column_id]
        .data_type
        .clone();

    let predicate_segment = parse_generative_segment(
        false,
        &mut VecDeque::from(predicate),
        &Some(output_type.clone()),
        project_context,
        None,
    )?;

    match output_type {
        TableDataTypeDescriptor::Enum(_) => {
            let final_predicate = predicate_segment.try_convert_enum(project_context)?;
            let simple_comparision_type = match comparision_type {
                Operator::Equality => SimpleComparisionType::Equals,
                Operator::Inequality => SimpleComparisionType::NotEquals,
                _ => {
                    return compile_err_token(
                        CompileErrorType::OnlyEqualsAndNotEqualsValidHere,
                        other_tokens.pop_front().unwrap(),
                    )
                }
            };
            Ok(FilterPredicate::EnumCompare(
                column,
                simple_comparision_type,
                final_predicate,
            ))
        }
        TableDataTypeDescriptor::String => {
            let final_predicate = predicate_segment.try_convert_string(project_context)?;
            let simple_comparision_type = match comparision_type {
                Operator::Equality => SimpleComparisionType::Equals,
                Operator::Inequality => SimpleComparisionType::NotEquals,
                _ => {
                    return compile_err_token(
                        CompileErrorType::OnlyEqualsAndNotEqualsValidHere,
                        other_tokens.pop_front().unwrap(),
                    )
                }
            };
            Ok(FilterPredicate::StringCompare(
                column,
                simple_comparision_type,
                final_predicate,
            ))
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
            Ok(FilterPredicate::UIntCompare(
                column,
                complex_comparision_type,
                final_predicate,
            ))
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
            Ok(FilterPredicate::IntCompare(
                column,
                complex_comparision_type,
                final_predicate,
            ))
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

    if !deque.is_empty() {
        // There are tokens left, meaning this expression
        // is not purely a column specifier.
        return false;
    }

    result.is_ok()
}

/// Begins parses any value expressions, whether that be a function
/// or a literal. May begin function processing with an optional parsing
/// context, for use within an inner segment.
fn parse_awaiting_value(
    current_token: Token,
    other_tokens: &mut VecDeque<Token>,
    target_data_type: &Option<DataTypeDescriptor>,
    project_context: &mut ProjectContext,
    automatic_recursion_end_token: Option<TokenType>,
) -> Result<(BuilderNode, Option<ParsingContext>), GenerativeProgramCompileError> {
    if let Some(DataTypeDescriptor::FilterPredicate) = target_data_type {
        // We have to return a filter predicate for the program to be valid,
        // so unlike the case with the table column specifier literal, we can
        // pull elements from the token queue without any checks or backups.
        let result = (
            BuilderNode::FilterPredicate(parse_filter_predicate_expression(
                current_token,
                other_tokens,
                project_context,
            )?),
            None,
        );

        if let Some(closing_token_type) = automatic_recursion_end_token {
            if closing_token_type != other_tokens.pop_front().unwrap().token_type {
                panic!()
            }
        }

        return Ok(result);
    }

    if let Some(closing_token_type) = automatic_recursion_end_token {
        // Since we are automatically doing this, the first token is still important
        other_tokens.push_front(current_token);
        let data_type = match target_data_type {
            Some(DataTypeDescriptor::TableDataType(v)) => Some(v.clone()),
            _ => None,
        };
        return Ok((
            parse_generative_segment(
                false,
                other_tokens,
                &data_type,
                project_context,
                Some(closing_token_type),
            )?,
            None,
        ));
    }

    match current_token.token_type {
        TokenType::OpenGroup(GroupType::Paren) => Ok((
            parse_generative_segment(
                false,
                other_tokens,
                &None,
                project_context,
                Some(TokenType::CloseGroup(GroupType::Paren)),
            )?,
            None,
        )),
        TokenType::Keyword(word) => match word {
            Keyword::Foreach => Ok((
                BuilderNode::CombinationNode(
                    super::node_builder::FunctionType::Foreach,
                    Vec::new(),
                    2,
                ),
                Some(ParsingContext::AwaitingFunctionBracket(VecDeque::from(
                    vec![DataTypeDescriptor::TableColumnSpecifier],
                ))),
            )),
            Keyword::Saved => Ok((
                BuilderNode::CombinationNode(
                    super::node_builder::FunctionType::Saved,
                    Vec::new(),
                    2,
                ),
                Some(ParsingContext::AwaitingFunctionBracket(VecDeque::from(
                    vec![
                        DataTypeDescriptor::TableDataType(TableDataTypeDescriptor::String),
                        DataTypeDescriptor::TableColumnSpecifier,
                    ],
                ))),
            )),
            Keyword::Mutate => Ok((
                BuilderNode::CombinationNode(
                    super::node_builder::FunctionType::Mutate,
                    Vec::new(),
                    2,
                ),
                Some(ParsingContext::AwaitingFunctionBracket(VecDeque::from(
                    vec![
                        DataTypeDescriptor::Range,
                        DataTypeDescriptor::Range,
                        DataTypeDescriptor::TableDataType(TableDataTypeDescriptor::UInt),
                    ],
                ))),
            )),
            Keyword::Filter | Keyword::Save => compile_err_token(
                CompileErrorType::SyntaxError(SyntaxErrorType::FunctionRequiresObject),
                current_token,
            ),
            _ => compile_err_token(
                CompileErrorType::SyntaxError(
                    SyntaxErrorType::InvalidKeywordDuringBlankStageParsing,
                ),
                current_token,
            ),
        },
        TokenType::NumericLiteral
        | TokenType::StringLiteral
        | TokenType::Symbol
        | TokenType::Operator(_) => Ok((
            BuilderNode::GenericLiteral(create_literal_node(
                current_token,
                other_tokens,
                project_context,
            )?),
            None,
        )),
        _ => compile_err_token(
            CompileErrorType::SyntaxError(SyntaxErrorType::InvalidTokenDuringBlankStageParsing),
            current_token,
        ),
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
        TokenType::StringLiteral => {
            let mut chars = current_token.token_contents.chars();
            chars.next();
            chars.next_back();
            Ok(UnderspecifiedLiteral::String(String::from(chars.as_str())))
        }
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

                    Ok(UnderspecifiedLiteral::TableColumnSpecifier(table_column))
                }
                Err(_) => {
                    // This is fine; it's not an enum with a
                    // table column specifier, so at this point we can
                    // just treat it like a numeric literal.
                    create_int_or_uint_literal(current_token)
                }
            }
        }
        // Int
        TokenType::Operator(Operator::Minus) => todo!(),
        // Enum
        TokenType::Operator(Operator::Colon) => {
            // It's an enum with a column specified
            // Or it could be a syntax error

            Ok(UnderspecifiedLiteral::TableColumnSpecifier(
                create_table_column_specifier(current_token, other_tokens, project_context)?,
            ))
        }
        _ => compile_err_token(
            CompileErrorType::SyntaxError(SyntaxErrorType::InvalidTokenDuringKeywordParsing),
            current_token,
        ),
    }
}

fn _create_enum_literal(
    contents: String,
    specifier: Option<TableColumnSpecifier>,
    current_column: ColumnSpecifier,
) -> Result<EnumSpecifier, GenerativeProgramCompileError> {
    let column = match specifier {
        Some(TableColumnSpecifier::Both(_, v)) | Some(TableColumnSpecifier::Column(v)) => v,
        Some(TableColumnSpecifier::Table(_)) => {
            return compile_err(CompileErrorType::OnlySpecifiedTable)
        }
        None => current_column,
    };

    let table = match specifier {
        Some(TableColumnSpecifier::Both(v, _)) | Some(TableColumnSpecifier::Table(v)) => Some(v),
        _ => None,
    };

    Ok(EnumSpecifier {
        name: contents,
        column,
        table,
    })
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
            if !other_tokens.is_empty() {
                let next_token = other_tokens.pop_front().unwrap();
                match next_token.token_type {
                    TokenType::NumericLiteral => match next_token.token_contents.parse::<usize>() {
                        Ok(column_id) => {
                            Ok(TableColumnSpecifier::Column(ColumnSpecifier { column_id }))
                        }
                        Err(error) => {
                            compile_err_token(CompileErrorType::IntParseError(error), current_token)
                        }
                    },
                    // Specifying column by name
                    TokenType::Symbol => Ok(TableColumnSpecifier::Column(ColumnSpecifier {
                        column_id: column_id_from_symbol(
                            project_context.table_id,
                            &next_token.token_contents,
                            project_context,
                        )?,
                    })),
                    _ => compile_err_token(
                        CompileErrorType::SyntaxError(
                            SyntaxErrorType::InvalidTokenDuringTableColumnSpecifierParsing(line!()),
                        ),
                        current_token,
                    ),
                }
            } else {
                compile_err_token(
                    CompileErrorType::SyntaxError(
                        SyntaxErrorType::InvalidTokenDuringTableColumnSpecifierParsing(line!()),
                    ),
                    current_token,
                )
            }
        }
        // Table only or both
        TokenType::NumericLiteral => match current_token.token_contents.parse::<usize>() {
            Ok(table_id) => {
                if !other_tokens.is_empty() {
                    let middle_token = other_tokens.pop_front().unwrap();
                    match middle_token.token_type {
                        TokenType::Operator(Operator::Colon) => {
                            if !other_tokens.is_empty() {
                                let next_token = other_tokens.pop_front().unwrap();
                                match next_token.token_type {
                                    TokenType::NumericLiteral => {
                                        match next_token.token_contents.parse::<usize>() {
                                            Ok(column_id) => Ok(TableColumnSpecifier::Both(
                                                TableSpecifier { table_id },
                                                ColumnSpecifier { column_id },
                                            )),
                                            Err(error) => compile_err_token(
                                                CompileErrorType::IntParseError(error),
                                                current_token,
                                            ),
                                        }
                                    }
                                    // Specifying column by name
                                    TokenType::Symbol => Ok(TableColumnSpecifier::Both(
                                        TableSpecifier { table_id },
                                        ColumnSpecifier {
                                            column_id: column_id_from_symbol(
                                                table_id,
                                                &next_token.token_contents,
                                                project_context,
                                            )?,
                                        },
                                    )),
                                    _ => {
                                        // In this case, the specifier is over so we return what
                                        // we have
                                        Ok(TableColumnSpecifier::Table(TableSpecifier { table_id }))
                                    }
                                }
                            } else {
                                // This is probably an error but not our responsibility
                                Ok(TableColumnSpecifier::Table(TableSpecifier { table_id }))
                            }
                        }
                        _ => compile_err_token(
                            CompileErrorType::SyntaxError(
                                SyntaxErrorType::InvalidTokenDuringTableColumnSpecifierParsing(
                                    line!(),
                                ),
                            ),
                            current_token,
                        ),
                    }
                } else {
                    compile_err_token(
                        CompileErrorType::SyntaxError(
                            SyntaxErrorType::InvalidTokenDuringTableColumnSpecifierParsing(line!()),
                        ),
                        current_token,
                    )
                }
            }
            Err(error) => compile_err_token(CompileErrorType::IntParseError(error), current_token),
        },
        _ => compile_err_token(
            CompileErrorType::SyntaxError(
                SyntaxErrorType::InvalidTokenDuringTableColumnSpecifierParsing(line!()),
            ),
            current_token,
        ),
    }
}

/// This functions turns a symbol which should contain a column id into a
/// column id to put into a descriptor
fn column_id_from_symbol(
    table: usize,
    symbol: &str,
    project_context: &mut ProjectContext,
) -> Result<usize, GenerativeProgramCompileError> {
    for (i, column) in project_context.all_descriptors[&table]
        .column_descriptors
        .iter()
        .enumerate()
    {
        if column.name.to_lowercase() == symbol.to_lowercase() {
            return Ok(i);
        }
    }

    compile_err(CompileErrorType::ColumnNotFound)
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
        Err(error) => return compile_err_token(CompileErrorType::IntParseError(error), token),
    };
    if value < 0 {
        // This probably shouldn't happen, as negative numbers should
        // appear as two tokens, but it never hurts to include.
        if value < i32::MIN as i64 {
            // Out of range
            compile_err_token(CompileErrorType::IntOutOfRange, token)
        } else {
            Ok(UnderspecifiedLiteral::Int(value as i32))
            // Int
        }
    } else {
        // TODO: Replace with a proper constants
        // It's a magic number right now because I don't have
        // internet and I don't where rust put its int limit
        // constants
        // i32 max
        if value > i32::MAX as i64 {
            // u32 max
            if value > u32::MAX as i64 {
                // Out of range
                compile_err_token(CompileErrorType::IntOutOfRange, token)
            } else {
                // UInt
                Ok(UnderspecifiedLiteral::UInt(value as u32))
            }
        } else {
            // Int or UInt
            Ok(UnderspecifiedLiteral::Number(value as u32, value as i32))
        }
    }
}
