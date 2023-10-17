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
    LiteralNode(String, ColumnSpecifier, TableSpecifier),
    ConversionNode(RangeNode),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum RangeNode {
    ForeachNode(TableSpecifier, ColumnSpecifier),
    FilterNode(Box<RangeNode>, Box<FilterPredicate>),
    Save(Box<RangeNode>, Box<StringNode>),
    Saved(Box<StringNode>, ColumnSpecifier),
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
    EnumCompare(ColumnSpecifier, SimpleComparisionType, EnumNode),
    StringCompare(ColumnSpecifier, SimpleComparisionType, StringNode),
    IntCompare(ColumnSpecifier, ComplexComparisionType, IntNode),
    UIntCompare(ColumnSpecifier, ComplexComparisionType, UIntNode),
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

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub struct RuntimeEnum {
    pub index: usize,
    pub table: TableSpecifier,
    pub column: ColumnSpecifier,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum GenerativeProgramExecutionOutput {
    String(Vec<String>),
    Int(Vec<i32>),
    UInt(Vec<u32>),
    Enum(Vec<RuntimeEnum>),
}

impl GenerativeProgramExecutionOutput {
    pub fn len(&self) -> usize {
        match self {
            GenerativeProgramExecutionOutput::String(v) => v.len(),
            GenerativeProgramExecutionOutput::Int(v) => v.len(),
            GenerativeProgramExecutionOutput::UInt(v) => v.len(),
            GenerativeProgramExecutionOutput::Enum(v) => v.len(),
        }
    }
}

impl GenerativeProgram {
    pub fn evaluate(
        &self,
        context: &mut ExecutionContext,
    ) -> Result<GenerativeProgramExecutionOutput, GenerativeProgramRuntimeError> {
        self.output_node.eval(context)
    }
}

impl OutputNode {
    pub fn eval(
        &self,
        context: &mut ExecutionContext,
    ) -> Result<GenerativeProgramExecutionOutput, GenerativeProgramRuntimeError> {
        match self {
            OutputNode::String(v) => Ok(GenerativeProgramExecutionOutput::String(v.eval(context)?)),
            OutputNode::Int(v) => Ok(GenerativeProgramExecutionOutput::Int(v.eval(context)?)),
            OutputNode::UInt(v) => Ok(GenerativeProgramExecutionOutput::UInt(v.eval(context)?)),
            OutputNode::Enum(v) => Ok(GenerativeProgramExecutionOutput::Enum(v.eval(context)?)),
        }
    }
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
            StringNode::ConversionNode(v) => {
                let range = v.eval(context)?;
                let column = range.column_id.unwrap();
                let mut result = Vec::new();

                for row in range.rows {
                    match row {
                        TableRow::PopulatedTableRow {
                            source: _,
                            descriptor: _,
                            contents,
                        } => {
                            let content = &contents[column];
                            match content {
                                TableContents::String(v) => result.push(v.clone()),
                                _ => todo!(),
                            }
                        }
                        // No range should have an unpopulated row
                        _ => unreachable!(),
                    }
                }

                Ok(result)
            }
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
                let (table, table_specifer) =
                    match &context.project.tables[table_specifier.table_id] {
                        Some(table) => (table.table_descriptor.clone(), *table_specifier),
                        None => return Err(GenerativeProgramRuntimeError::TableNotFound),
                    };
                let data_type = &table.column_descriptors[column_specifier.column_id].data_type;
                let values = match data_type {
                    crate::manual_ux::table::TableDataTypeDescriptor::Enum(v) => v,
                    // Assuming that the creation of the node is done correctly,
                    // this will never happen and will be unreachable
                    _ => unreachable!(),
                };
                match values
                    .iter()
                    .position(|elem| elem.to_lowercase() == key.to_lowercase())
                {
                    Some(index) => Ok(vec![RuntimeEnum {
                        index,
                        table: table_specifer,
                        column: *column_specifier,
                    }]),
                    None => Err(GenerativeProgramRuntimeError::EnumNotFound),
                }
            }
            EnumNode::ConversionNode(v) => {
                let range = v.eval(context)?;
                let column = range.column_id.unwrap();
                let mut result = Vec::new();

                for row in range.rows {
                    match row {
                        TableRow::PopulatedTableRow {
                            source: _,
                            descriptor: _,
                            contents,
                        } => {
                            let content = &contents[column];
                            match content {
                                TableContents::Enum(v) => result.push(v.clone()),
                                _ => todo!(),
                            }
                        }
                        // No range should have an unpopulated row
                        _ => unreachable!(),
                    }
                }

                Ok(result)
            }
        }
    }
}

impl RangeNode {
    pub fn eval(
        &self,
        context: &mut ExecutionContext,
    ) -> Result<Range, GenerativeProgramRuntimeError> {
        match self {
            RangeNode::ForeachNode(table, column) => {
                let table = match &context.project.tables[table.table_id] {
                    Some(v) => v,
                    None => return Err(GenerativeProgramRuntimeError::TableNotFound),
                };

                Ok(Range {
                    rows: table.table_rows.clone(),
                    column_id: Some(column.column_id),
                })
            }
            RangeNode::FilterNode(range, predicate) => {
                let mut result = range.eval(context)?;
                let mut new_range: Vec<TableRow> = Vec::with_capacity(result.rows.len());
                for x in result.rows {
                    if (&predicate).check(&x, context)? {
                        new_range.push(x);
                    }
                }
                result.rows = new_range;

                return Ok(result);
            }
            RangeNode::Save(range, key) => {
                let evaluated = range.to_owned().eval(context)?;
                // TODO: Work out vector string results
                let key = key.eval(context)?[0].clone();
                context.saved_ranges.insert(key, evaluated.clone());
                return Ok(evaluated);
            }
            RangeNode::Saved(key, column) => {
                let key_value = &key.eval(context)?[0];
                let mut result = context.saved_ranges[key_value].clone();
                result.column_id = Some(column.column_id);

                return Ok(result);
            }
        }
    }
}

impl FilterPredicate {
    pub fn check(
        &self,
        row: &TableRow,
        context: &mut ExecutionContext,
    ) -> Result<bool, GenerativeProgramRuntimeError> {
        // Check data type of input
        let (descriptor, contents) = match row {
            TableRow::PopulatedTableRow {
                descriptor,
                contents,
                ..
            } => (descriptor, contents),
            TableRow::UnpopulatedTableRow { .. } => {
                return Err(GenerativeProgramRuntimeError::OutOfOrderExecution);
            }
        };

        let column_id = match self {
            FilterPredicate::EnumCompare(column, _, _) => column.column_id,
            FilterPredicate::StringCompare(column, _, _) => column.column_id,
            FilterPredicate::IntCompare(column, _, _) => column.column_id,
            FilterPredicate::UIntCompare(column, _, _) => column.column_id,
        };

        let input_data_type = &descriptor.column_descriptors[column_id].data_type;

        // Crazy multi-level match statement
        match self {
            FilterPredicate::EnumCompare(_, comp_type, node) => match input_data_type {
                TableDataTypeDescriptor::Enum(_) => match contents[column_id] {
                    TableContents::Enum(v) => {
                        let check_value = node.eval(context)?;
                        if check_value.len() == 1 {
                            match comp_type {
                                SimpleComparisionType::Equals => Ok(v == check_value[0]),
                                SimpleComparisionType::NotEquals => Ok(v != check_value[0]),
                            }
                        } else {
                            // Check to see if it matches with any of the given values
                            todo!()
                        }
                    }
                    // Assuming that the creation of the descriptor is done correctly,
                    // this will never happen and will be unreachable
                    _ => unreachable!(),
                },
                _ => return Err(GenerativeProgramRuntimeError::MismatchedRangeLengths),
            },
            FilterPredicate::StringCompare(_, comp_type, node) => match input_data_type {
                TableDataTypeDescriptor::String => match &contents[column_id] {
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
            FilterPredicate::IntCompare(_, _comp_type, _node) => match input_data_type {
                TableDataTypeDescriptor::Int => match contents[column_id] {
                    TableContents::Int(_v) => todo!(),
                    // Assuming that the creation of the descriptor is done correctly,
                    // this will never happen and will be unreachable
                    _ => unreachable!(),
                },
                _ => return Err(GenerativeProgramRuntimeError::MismatchedRangeLengths),
            },
            FilterPredicate::UIntCompare(_, _comp_type, _node) => match input_data_type {
                TableDataTypeDescriptor::UInt => match contents[column_id] {
                    TableContents::UInt(_v) => todo!(),
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
fn _enforce_single_usize(input: Vec<usize>) -> Result<usize, GenerativeProgramRuntimeError> {
    if input.len() == 1 {
        Ok(input[0])
    } else {
        Err(GenerativeProgramRuntimeError::MismatchedRangeLengths)
    }
}

fn _enforce_single_i32(input: Vec<i32>) -> Result<i32, GenerativeProgramRuntimeError> {
    if input.len() == 1 {
        Ok(input[0])
    } else {
        Err(GenerativeProgramRuntimeError::MismatchedRangeLengths)
    }
}

fn _enforce_single_u32(input: Vec<u32>) -> Result<u32, GenerativeProgramRuntimeError> {
    if input.len() == 1 {
        Ok(input[0])
    } else {
        Err(GenerativeProgramRuntimeError::MismatchedRangeLengths)
    }
}
