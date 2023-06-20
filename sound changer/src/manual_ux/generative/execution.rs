use super::super::table::*;
use super::*;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExecutionContext<'a> {
    pub saved_ranges: HashMap<String, Range>,
    pub table_descriptor: Rc<TableDescriptor>,
    pub table_specifer: TableSpecifier,
    pub project: &'a Project,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum OutputNode {
    String(StringNode),
    Int(IntNode),
    UInt(UIntNode),
    Enum(EnumNode),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum StringNode {
    LiteralNode(String),
    AdditionNode(Box<StringNode>, Box<StringNode>),
    ConversionNode(RangeNode),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum IntNode {
    LiteralNode(i32),
    AdditionNode(Box<IntNode>, Box<IntNode>),
    ConversionNode(RangeNode),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum UIntNode {
    LiteralNode(u32),
    AdditionNode(Box<UIntNode>, Box<UIntNode>),
    ConversionNode(RangeNode),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum EnumNode {
    LiteralNode(String, ColumnSpecifier, Option<TableSpecifier>),
    ConversionNode(RangeNode),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum RangeNode {
    ForeachNode(TableSpecifier, Option<ColumnSpecifier>),
    FilterNode(Box<RangeNode>, ColumnSpecifier, Box<FilterPredicate>),
    Save(Box<RangeNode>, String),
    Saved(String, Option<ColumnSpecifier>),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Range {
    pub rows: Vec<TableRow>,
    pub column_id: Option<usize>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub struct TableSpecifier {
    pub table_id: usize,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub struct ColumnSpecifier {
    pub column_id: usize,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FilterPredicate {
    EnumCompare(SimpleComparisionType, EnumNode),
    StringCompare(SimpleComparisionType, StringNode),
    IntCompare(ComplexComparisionType, IntNode),
    UIntCompare(ComplexComparisionType, UIntNode),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SimpleComparisionType {
    Equals,
    NotEquals,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ComplexComparisionType {
    Equals,
    NotEquals,
    Greater,
    Less,
    GreaterEquals,
    LessEquals,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RuntimeEnum {
    index: usize,
    table: TableSpecifier,
    column: ColumnSpecifier,
}

impl StringNode {
    pub fn eval(
        &self,
        context: &mut ExecutionContext,
    ) -> Result<Vec<String>, GenerativeProgramRuntimeError> {
        match self {
            StringNode::LiteralNode(contents) => Ok(vec![contents.clone()]),
            StringNode::AdditionNode(a, b) => {
                let mut operand1 = a.eval(context)?;
                let mut operand2 = b.eval(context)?;

                // TODO: Work out DRY here
                if operand1.len() != operand2.len() {
                    if operand1.len() == 1 {
                        let mut i = 0;
                        while i < operand2.len() {
                            operand2[i] += &operand1[0];
                            i += 1;
                        }
                        return Ok(operand2);
                    }
                    if operand2.len() == 1 {
                        let mut i = 0;
                        while i < operand1.len() {
                            operand1[i] += &operand2[0];
                            i += 1;
                        }
                        return Ok(operand1);
                    }
                    return Err(GenerativeProgramRuntimeError::MismatchedRangeLengths);
                }

                let mut i = 0;
                while i < operand1.len() {
                    operand1[i] += &operand2[i];
                    i += 1;
                }
                return Ok(operand1);
            }
            StringNode::ConversionNode(_) => todo!(),
        }
    }
}

impl IntNode {
    pub fn eval(
        &self,
        context: &mut ExecutionContext,
    ) -> Result<Vec<i32>, GenerativeProgramRuntimeError> {
        match self {
            IntNode::LiteralNode(contents) => Ok(vec![*contents]),
            IntNode::AdditionNode(a, b) => {
                let mut operand1 = a.eval(context)?;
                let mut operand2 = b.eval(context)?;

                // TODO: Work out DRY here
                if operand1.len() != operand2.len() {
                    if operand1.len() == 1 {
                        let mut i = 0;
                        while i < operand2.len() {
                            operand2[i] += &operand1[0];
                            i += 1;
                        }
                        return Ok(operand2);
                    }
                    if operand2.len() == 1 {
                        let mut i = 0;
                        while i < operand1.len() {
                            operand1[i] += &operand2[0];
                            i += 1;
                        }
                        return Ok(operand1);
                    }
                    return Err(GenerativeProgramRuntimeError::MismatchedRangeLengths);
                }

                let mut i = 0;
                while i < operand1.len() {
                    operand1[i] += &operand2[i];
                    i += 1;
                }
                return Ok(operand1);
            }
            IntNode::ConversionNode(_) => todo!(),
        }
    }
}

impl UIntNode {
    pub fn eval(
        &self,
        context: &mut ExecutionContext,
    ) -> Result<Vec<u32>, GenerativeProgramRuntimeError> {
        match self {
            UIntNode::LiteralNode(contents) => Ok(vec![*contents]),
            UIntNode::AdditionNode(a, b) => {
                let mut operand1 = a.eval(context)?;
                let mut operand2 = b.eval(context)?;

                if operand1.len() != operand2.len() {
                    if operand1.len() == 1 {
                        let mut i = 0;
                        while i < operand2.len() {
                            operand2[i] += &operand1[0];
                            i += 1;
                        }
                        return Ok(operand2);
                    }
                    if operand2.len() == 1 {
                        let mut i = 0;
                        while i < operand1.len() {
                            operand1[i] += &operand2[0];
                            i += 1;
                        }
                        return Ok(operand1);
                    }
                    return Err(GenerativeProgramRuntimeError::MismatchedRangeLengths);
                }

                let mut i = 0;
                while i < operand1.len() {
                    operand1[i] += &operand2[i];
                    i += 1;
                }
                return Ok(operand1);
            }
            UIntNode::ConversionNode(_) => todo!(),
        }
    }
}

impl EnumNode {
    pub fn eval(
        &self,
        context: &mut ExecutionContext,
    ) -> Result<Vec<RuntimeEnum>, GenerativeProgramRuntimeError> {
        match self {
            EnumNode::LiteralNode(key, column_specifier, table_specifier) => {
                let (table, table_specifer) = match table_specifier {
                    Some(index) => match &context.project.tables[index.table_id] {
                        Some(table) => (table.table_descriptor.clone(), *index),
                        None => return Err(GenerativeProgramRuntimeError::TableNotFound),
                    },
                    None => (context.table_descriptor.clone(), context.table_specifer),
                };
                let data_type = &table.column_descriptors[column_specifier.column_id].data_type;
                let values = match data_type {
                    crate::manual_ux::table::TableDataTypeDescriptor::Enum(v) => v,
                    // Assuming that the creation of the node is done correctly,
                    // this will never happen and will be unreachable
                    _ => unreachable!(),
                };
                match values.iter().position(|elem| elem == key) {
                    Some(index) => Ok(vec![RuntimeEnum {
                        index,
                        table: table_specifer,
                        column: *column_specifier,
                    }]),
                    None => Err(GenerativeProgramRuntimeError::EnumNotFound),
                }
            }
            EnumNode::ConversionNode(_) => todo!(),
        }
    }
}

impl RangeNode {
    pub fn eval(
        &self,
        context: &mut ExecutionContext,
    ) -> Result<Range, GenerativeProgramRuntimeError> {
        match self {
            RangeNode::ForeachNode(_, _) => todo!(),
            RangeNode::FilterNode(range, column, predicate) => {
                let mut result = range.eval(context)?;
                let mut new_range: Vec<TableRow> = Vec::with_capacity(result.rows.len());
                for x in result.rows {
                    if (&predicate).check(&x, &column, context)? {
                        new_range.push(x);
                    }
                }
                result.rows = new_range;

                return Ok(result);
            }
            RangeNode::Save(range, key) => {
                let evaluated = range.to_owned().eval(context)?;
                context.saved_ranges.insert(key.clone(), evaluated.clone());
                return Ok(evaluated);
            }
            RangeNode::Saved(key, column) => {
                let mut result = context.saved_ranges[key].clone();
                match column {
                    Some(v) => result.column_id = Some(v.column_id),
                    None => (),
                }

                return Ok(result);
            }
        }
    }
}

impl FilterPredicate {
    pub fn check(
        &self,
        row: &TableRow,
        column: &ColumnSpecifier,
        context: &mut ExecutionContext,
    ) -> Result<bool, GenerativeProgramRuntimeError> {
        // Check data type of input
        let (descriptor, contents) = match row {
            TableRow::PopulatedTableRow {
                descriptor,
                contents,
                ..
            } => (descriptor, contents),
            TableRow::UnpopulatedTableRow { descriptor, .. } => {
                return Err(GenerativeProgramRuntimeError::OutOfOrderExecution);
            }
        };
        let input_data_type = &descriptor.column_descriptors[column.column_id].data_type;

        // Crazy multi-level match statement
        match self {
            FilterPredicate::EnumCompare(comp_type, node) => match input_data_type {
                TableDataTypeDescriptor::Enum(_) => match contents[column.column_id] {
                    TableContents::Enum(v) => todo!(),
                    // Assuming that the creation of the descriptor is done correctly,
                    // this will never happen and will be unreachable
                    _ => unreachable!(),
                },
                _ => return Err(GenerativeProgramRuntimeError::MismatchedRangeLengths),
            },
            FilterPredicate::StringCompare(comp_type, node) => match input_data_type {
                TableDataTypeDescriptor::String => match &contents[column.column_id] {
                    TableContents::String(v) => {
                        let eval = enforce_single_string(node.eval(context)?)?;
                        return match comp_type {
                            SimpleComparisionType::Equals => Ok(eval == *v),
                            SimpleComparisionType::NotEquals => Ok(eval != *v),
                        };
                    }
                    // Assuming that the creation of the descriptor is done correctly,
                    // this will never happen and will be unreachable
                    _ => unreachable!(),
                },
                _ => return Err(GenerativeProgramRuntimeError::MismatchedRangeLengths),
            },
            FilterPredicate::IntCompare(comp_type, node) => match input_data_type {
                TableDataTypeDescriptor::Int => match contents[column.column_id] {
                    TableContents::Int(v) => todo!(),
                    // Assuming that the creation of the descriptor is done correctly,
                    // this will never happen and will be unreachable
                    _ => unreachable!(),
                },
                _ => return Err(GenerativeProgramRuntimeError::MismatchedRangeLengths),
            },
            FilterPredicate::UIntCompare(comp_type, node) => match input_data_type {
                TableDataTypeDescriptor::UInt => match contents[column.column_id] {
                    TableContents::UInt(v) => todo!(),
                    // Assuming that the creation of the descriptor is done correctly,
                    // this will never happen and will be unreachable
                    _ => unreachable!(),
                },
                _ => return Err(GenerativeProgramRuntimeError::MismatchedRangeLengths),
            },
        }
    }
}

fn enforce_single_string(input: Vec<String>) -> Result<String, GenerativeProgramRuntimeError> {
    if input.len() == 1 {
        Ok(input[0].clone())
    } else {
        Err(GenerativeProgramRuntimeError::MismatchedRangeLengths)
    }
}

// TODO: Figure out exactly how to represents enums
fn enforce_single_usize(input: Vec<usize>) -> Result<usize, GenerativeProgramRuntimeError> {
    if input.len() == 1 {
        Ok(input[0])
    } else {
        Err(GenerativeProgramRuntimeError::MismatchedRangeLengths)
    }
}

fn enforce_single_i32(input: Vec<i32>) -> Result<i32, GenerativeProgramRuntimeError> {
    if input.len() == 1 {
        Ok(input[0])
    } else {
        Err(GenerativeProgramRuntimeError::MismatchedRangeLengths)
    }
}

fn enforce_single_u32(input: Vec<u32>) -> Result<u32, GenerativeProgramRuntimeError> {
    if input.len() == 1 {
        Ok(input[0])
    } else {
        Err(GenerativeProgramRuntimeError::MismatchedRangeLengths)
    }
}