use crate::applicator::from_string;
use crate::constructor::construct;
use crate::data::{to_string, Program};
use crate::io;

use super::super::table::*;
use super::*;

pub struct ExecutionContext<'a> {
    pub saved_ranges: HashMap<String, Range>,
    pub table_descriptor: Rc<TableDescriptor>,
    pub table_specifer: TableSpecifier,
    pub base_path: String,
    pub project: &'a Project,
    pub programs: &'a mut HashMap<String, Program>,
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
    SoundChangeNode(Box<StringNode>, Box<StringNode>),
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
    Mutate(Box<RangeNode>, Box<RangeNode>, Box<UIntNode>),
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

    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
                Ok(operand1)
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
            StringNode::SoundChangeNode(source, program) => {
                let program_name = enforce_single_string(program.eval(context)?)?;
                load_program_if_not_loaded(&program_name, context)?;
                let inputs = source.eval(context)?;

                Ok(apply_sc(&program_name, inputs, context)?)
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
                Ok(operand1)
            }
            IntNode::ConversionNode(v) => {
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
                                TableContents::Int(v) => result.push(*v),
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
                Ok(operand1)
            }
            UIntNode::ConversionNode(v) => {
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
                                TableContents::UInt(v) => result.push(*v),
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
                                TableContents::Enum(v) => result.push(*v),
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
                    if predicate.check(&x, context)? {
                        new_range.push(x);
                    }
                }
                result.rows = new_range;

                Ok(result)
            }
            RangeNode::Save(range, key) => {
                let evaluated = range.to_owned().eval(context)?;
                // TODO: Work out vector string results
                let key = key.eval(context)?[0].clone();
                context.saved_ranges.insert(key, evaluated.clone());
                Ok(evaluated)
            }
            RangeNode::Saved(key, column) => {
                let key_value = &key.eval(context)?[0];
                let mut result = context.saved_ranges[key_value].clone();
                result.column_id = Some(column.column_id);

                Ok(result)
            }
            RangeNode::Mutate(a, b, mode) => {
                // TODO: More reliable take mode
                mutate(a.eval(context)?, b.eval(context)?, mode.eval(context)?[0])
            }
        }
    }
}

fn mutate(a: Range, b: Range, mode: u32) -> Result<Range, GenerativeProgramRuntimeError> {
    println!("{:?}, {:?}", a.column_id, b.column_id);
    let mut results = Vec::new();
    for a_comp in &a.rows {
        for b_comp in &b.rows {
            let a_val = a_comp.unwrap(a.column_id)?;
            let b_val = b_comp.unwrap(b.column_id)?;
            match mode {
                // Take a
                0 => results.push(a_val),
                // Take b
                1 => results.push(b_val),
                // Component wise addition
                2 => results.push(add(a_val, b_val)?),
                _ => todo!(),
            }
        }
    }

    if results.is_empty() {
        todo!()
    }
    // TODO: Check for uniformity in output data type.
    let data_type = match results[0] {
        TableContents::Enum(_) => todo!(),
        TableContents::String(_) => TableDataTypeDescriptor::String,
        TableContents::UInt(_) => TableDataTypeDescriptor::UInt,
        TableContents::Int(_) => TableDataTypeDescriptor::Int,
    };

    let descriptor = Rc::new(TableDescriptor {
        column_descriptors: vec![TableColumnDescriptor {
            name: String::from("AUTO GENERATED FROM MUTATE"),
            data_type,
        }],
    });
    let mut rows = Vec::new();
    for result in results {
        rows.push(TableRow::PopulatedTableRow {
            source: PopulatedTableRowSource::MUTATE,
            descriptor: descriptor.clone(),
            contents: vec![result],
        });
    }

    Ok(Range {
        rows,
        column_id: Some(0),
    })
}

// TODO: Turn into operator overload?
fn add(a: TableContents, b: TableContents) -> Result<TableContents, GenerativeProgramRuntimeError> {
    match a {
        TableContents::String(av) => match b {
            TableContents::String(bv) => Ok(TableContents::String(av + &bv)),
            _ => todo!(),
        },
        TableContents::UInt(av) => match b {
            TableContents::UInt(bv) => Ok(TableContents::UInt(av + bv)),
            _ => todo!(),
        },
        TableContents::Int(av) => match b {
            TableContents::Int(bv) => Ok(TableContents::Int(av + bv)),
            _ => todo!(),
        },
        TableContents::Enum(_) => todo!(),
    }
}

impl TableRow {
    fn unwrap(
        &self,
        column: Option<usize>,
    ) -> Result<TableContents, GenerativeProgramRuntimeError> {
        match self {
            TableRow::PopulatedTableRow {
                source: _,
                descriptor: _,
                contents,
            } => match column {
                Some(v) => Ok(contents[v].clone()),
                None => todo!(),
            },
            TableRow::UnpopulatedTableRow {
                procedure: _,
                descriptor: _,
            } => todo!(),
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
                _ => Err(GenerativeProgramRuntimeError::MismatchedRangeLengths),
            },
            FilterPredicate::StringCompare(_, comp_type, node) => match input_data_type {
                TableDataTypeDescriptor::String => match &contents[column_id] {
                    TableContents::String(v) => {
                        let eval = enforce_single_string(node.eval(context)?)?;
                        match comp_type {
                            SimpleComparisionType::Equals => Ok(eval == *v),
                            SimpleComparisionType::NotEquals => Ok(eval != *v),
                        }
                    }
                    // Assuming that the creation of the descriptor is done correctly,
                    // this will never happen and will be unreachable
                    _ => unreachable!(),
                },
                _ => Err(GenerativeProgramRuntimeError::MismatchedRangeLengths),
            },
            FilterPredicate::IntCompare(_, comp_type, node) => match input_data_type {
                TableDataTypeDescriptor::Int => match contents[column_id] {
                    TableContents::Int(v) => {
                        let eval = enforce_single_i32(node.eval(context)?)?;
                        // TODO: Check direction on < <= > >=
                        match comp_type {
                            ComplexComparisionType::Equals => Ok(eval == v),
                            ComplexComparisionType::NotEquals => Ok(eval != v),
                            ComplexComparisionType::Greater => Ok(eval > v),
                            ComplexComparisionType::GreaterEquals => Ok(eval >= v),
                            ComplexComparisionType::Less => Ok(eval < v),
                            ComplexComparisionType::LessEquals => Ok(eval <= v),
                        }
                    }
                    // Assuming that the creation of the descriptor is done correctly,
                    // this will never happen and will be unreachable
                    _ => unreachable!(),
                },
                _ => Err(GenerativeProgramRuntimeError::MismatchedRangeLengths),
            },
            FilterPredicate::UIntCompare(_, comp_type, node) => match input_data_type {
                TableDataTypeDescriptor::UInt => match contents[column_id] {
                    TableContents::UInt(v) => {
                        let eval = enforce_single_u32(node.eval(context)?)?;
                        match comp_type {
                            ComplexComparisionType::Equals => Ok(eval == v),
                            ComplexComparisionType::NotEquals => Ok(eval != v),
                            ComplexComparisionType::Greater => Ok(eval > v),
                            ComplexComparisionType::GreaterEquals => Ok(eval >= v),
                            ComplexComparisionType::Less => Ok(eval < v),
                            ComplexComparisionType::LessEquals => Ok(eval <= v),
                        }
                    }
                    // Assuming that the creation of the descriptor is done correctly,
                    // this will never happen and will be unreachable
                    _ => unreachable!(),
                },
                _ => Err(GenerativeProgramRuntimeError::MismatchedRangeLengths),
            },
        }
    }
}

fn load_program_if_not_loaded(
    name: &String,
    context: &mut ExecutionContext,
) -> Result<(), GenerativeProgramRuntimeError> {
    if context.programs.contains_key(name) {
        return Ok(());
    }

    let path_str = &format!("{}\\{}.lsc", context.base_path, name);
    let contents = match io::load_from_file(path_str, false) {
        Ok(v) => v,
        // TODO: Pass along error information
        Err(_) => return Err(GenerativeProgramRuntimeError::IOError),
    };
    let program = match construct(&contents) {
        Ok(v) => v,
        // TODO: Pass along error information
        Err(_) => return Err(GenerativeProgramRuntimeError::SoundChangeCompileError),
    };

    context.programs.insert(name.clone(), program);

    Ok(())
}

// TODO: Error handling
fn apply_sc(
    program_name: &String,
    inputs: Vec<String>,
    context: &mut ExecutionContext,
) -> Result<Vec<String>, GenerativeProgramRuntimeError> {
    let program = context.programs.get(program_name).unwrap();
    let mut results = Vec::with_capacity(inputs.len());

    for input in inputs {
        let converted_string = from_string(program, &input).unwrap();
        let changed_word = program.apply(converted_string).unwrap();
        let result = to_string(program, changed_word).unwrap();
        results.push(result);
    }

    Ok(results)
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
