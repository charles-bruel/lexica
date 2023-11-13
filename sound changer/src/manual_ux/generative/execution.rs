use serde::{Deserialize, Serialize};

use crate::{
    io,
    manual_ux::project::Project,
    sc::{
        applicator::from_string,
        constructor::construct,
        data::{to_string, Program},
    },
};

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

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub struct TableSpecifier {
    pub table_id: usize,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug, Copy)]
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

impl SimpleComparisionType {
    pub fn compare<T>(&self, a: &T, b: &T) -> bool
    where
        T: PartialEq,
    {
        match self {
            SimpleComparisionType::Equals => a == b,
            SimpleComparisionType::NotEquals => a != b,
        }
    }
}

impl ComplexComparisionType {
    pub fn compare<T>(&self, a: &T, b: &T) -> bool
    where
        T: PartialEq + PartialOrd,
    {
        match self {
            ComplexComparisionType::Equals => a == b,
            ComplexComparisionType::NotEquals => a != b,
            ComplexComparisionType::Greater => a > b,
            ComplexComparisionType::GreaterEquals => a >= b,
            ComplexComparisionType::Less => a < b,
            ComplexComparisionType::LessEquals => a <= b,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug, Copy)]
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

/// Trait for generically adding two data types.
/// Existing generics are insufficient because there is no default
/// implementation of add for String and String (or &String)
trait DataAdd<T> {
    fn add(&self, other: &T) -> T;
}

impl DataAdd<String> for String {
    fn add(&self, other: &String) -> String {
        self.clone() + other
    }
}

impl DataAdd<u32> for u32 {
    fn add(&self, other: &u32) -> u32 {
        self + other
    }
}

impl DataAdd<i32> for i32 {
    fn add(&self, other: &i32) -> i32 {
        self + other
    }
}

fn add_vecs<T: DataAdd<T>>(
    mut operand1: Vec<T>,
    operand2: Vec<T>,
) -> Result<Vec<T>, GenerativeProgramRuntimeError> {
    // TODO: Work out DRY here
    if operand1.len() != operand2.len() {
        let (single_operand, mut multi_operand) = if operand1.len() == 1 {
            (&operand1[0], operand2)
        } else if operand2.len() == 1 {
            (&operand2[0], operand1)
        } else {
            return runtime_err(RuntimeErrorType::MismatchedRangeLengths);
        };

        let mut i = 0;
        while i < multi_operand.len() {
            multi_operand[i] = multi_operand[i].add(single_operand);
            i += 1;
        }
        return Ok(multi_operand);
    }

    let mut i = 0;
    while i < operand1.len() {
        operand1[i] = operand1[i].add(&operand2[i]);
        i += 1;
    }
    Ok(operand1)
}

impl From<TableContents> for Result<RuntimeEnum, GenerativeProgramRuntimeError> {
    fn from(val: TableContents) -> Self {
        match val {
            TableContents::Enum(v) => Ok(v),
            _ => runtime_err(RuntimeErrorType::TypeMismatch),
        }
    }
}

impl From<TableContents> for Result<String, GenerativeProgramRuntimeError> {
    fn from(val: TableContents) -> Self {
        match val {
            TableContents::String(v) => Ok(v),
            _ => runtime_err(RuntimeErrorType::TypeMismatch),
        }
    }
}

impl From<TableContents> for Result<u32, GenerativeProgramRuntimeError> {
    fn from(val: TableContents) -> Self {
        match val {
            TableContents::UInt(v) => Ok(v),
            _ => runtime_err(RuntimeErrorType::TypeMismatch),
        }
    }
}

impl From<TableContents> for Result<i32, GenerativeProgramRuntimeError> {
    fn from(val: TableContents) -> Self {
        match val {
            TableContents::Int(v) => Ok(v),
            _ => runtime_err(RuntimeErrorType::TypeMismatch),
        }
    }
}

fn convert<T>(range: Range) -> Result<Vec<T>, GenerativeProgramRuntimeError>
where
    TableContents: Into<Result<T, GenerativeProgramRuntimeError>>,
{
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
                result.push(std::convert::Into::<
                    Result<T, GenerativeProgramRuntimeError>,
                >::into(content.clone())?)
            }
            // No range should have an unpopulated row
            _ => unreachable!(),
        }
    }

    Ok(result)
}

impl StringNode {
    pub fn eval(
        &self,
        context: &mut ExecutionContext,
    ) -> Result<Vec<String>, GenerativeProgramRuntimeError> {
        match self {
            StringNode::LiteralNode(contents) => Ok(vec![contents.clone()]),
            StringNode::AdditionNode(a, b) => add_vecs(a.eval(context)?, b.eval(context)?),
            StringNode::ConversionNode(v) => convert(v.eval(context)?),
            StringNode::SoundChangeNode(source, program) => {
                let program_name = enforce_single(program.eval(context)?)?;
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
            IntNode::AdditionNode(a, b) => add_vecs(a.eval(context)?, b.eval(context)?),
            IntNode::ConversionNode(v) => convert(v.eval(context)?),
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
            UIntNode::AdditionNode(a, b) => add_vecs(a.eval(context)?, b.eval(context)?),
            UIntNode::ConversionNode(v) => convert(v.eval(context)?),
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
                        None => return runtime_err(RuntimeErrorType::TableNotFound),
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
                    None => runtime_err(RuntimeErrorType::EnumNotFound),
                }
            }
            EnumNode::ConversionNode(v) => convert(v.eval(context)?),
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
                    None => return runtime_err(RuntimeErrorType::TableNotFound),
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
                mutate(
                    a.eval(context)?,
                    b.eval(context)?,
                    mode.eval(context)?[0],
                    context,
                )
            }
        }
    }
}

fn mutate(
    a: Range,
    b: Range,
    mode: u32,
    context: &ExecutionContext,
) -> Result<Range, GenerativeProgramRuntimeError> {
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
    let data_type = results[0].to_data_type(context);

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
            _ => panic!(),
        },
        TableContents::UInt(av) => match b {
            TableContents::UInt(bv) => Ok(TableContents::UInt(av + bv)),
            _ => panic!(),
        },
        TableContents::Int(av) => match b {
            TableContents::Int(bv) => Ok(TableContents::Int(av + bv)),
            _ => panic!(),
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
                None => panic!(),
            },
            TableRow::UnpopulatedTableRow {
                procedure: _,
                descriptor: _,
            } => panic!(),
        }
    }
}

impl FilterPredicate {
    pub fn get_column_id(&self) -> usize {
        match self {
            FilterPredicate::EnumCompare(column, _, _) => column.column_id,
            FilterPredicate::StringCompare(column, _, _) => column.column_id,
            FilterPredicate::IntCompare(column, _, _) => column.column_id,
            FilterPredicate::UIntCompare(column, _, _) => column.column_id,
        }
    }
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
                return runtime_err(RuntimeErrorType::OutOfOrderExecution);
            }
        };

        let column_id = self.get_column_id();

        let input_data_type = &descriptor.column_descriptors[column_id].data_type;

        // Crazy multi-level match statement
        match self {
            FilterPredicate::EnumCompare(_, comp_type, node) => match input_data_type {
                TableDataTypeDescriptor::Enum(_) => match contents[column_id] {
                    TableContents::Enum(v) => {
                        let check_value = node.eval(context)?;
                        if check_value.len() == 1 {
                            Ok(comp_type.compare(&v, &check_value[0]))
                        } else {
                            // Check to see if it matches with any of the given values
                            todo!()
                        }
                    }
                    // Assuming that the creation of the descriptor is done correctly,
                    // this will never happen and will be unreachable
                    _ => unreachable!(),
                },
                _ => runtime_err(RuntimeErrorType::MismatchedRangeLengths),
            },
            FilterPredicate::StringCompare(_, comp_type, node) => match input_data_type {
                TableDataTypeDescriptor::String => match &contents[column_id] {
                    TableContents::String(v) => {
                        let eval: String = enforce_single(node.eval(context)?)?;
                        Ok(comp_type.compare(&eval, v))
                    }
                    // Assuming that the creation of the descriptor is done correctly,
                    // this will never happen and will be unreachable
                    _ => unreachable!(),
                },
                _ => runtime_err(RuntimeErrorType::MismatchedRangeLengths),
            },
            FilterPredicate::IntCompare(_, comp_type, node) => match input_data_type {
                TableDataTypeDescriptor::Int => match contents[column_id] {
                    TableContents::Int(v) => {
                        let eval = enforce_single(node.eval(context)?)?;
                        // TODO: Check direction on < <= > >=
                        Ok(comp_type.compare(&eval, &v))
                    }
                    // Assuming that the creation of the descriptor is done correctly,
                    // this will never happen and will be unreachable
                    _ => unreachable!(),
                },
                _ => runtime_err(RuntimeErrorType::MismatchedRangeLengths),
            },
            FilterPredicate::UIntCompare(_, comp_type, node) => match input_data_type {
                TableDataTypeDescriptor::UInt => match contents[column_id] {
                    TableContents::UInt(v) => {
                        let eval = enforce_single(node.eval(context)?)?;
                        Ok(comp_type.compare(&eval, &v))
                    }
                    // Assuming that the creation of the descriptor is done correctly,
                    // this will never happen and will be unreachable
                    _ => unreachable!(),
                },
                _ => runtime_err(RuntimeErrorType::MismatchedRangeLengths),
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

    let path_str = &format!("{}/{}.lsc", context.base_path, name);
    let contents = match io::load_from_file(path_str, false) {
        Ok(v) => v,
        // TODO: Pass along error information
        Err(err) => return runtime_err(RuntimeErrorType::IOError(err)),
    };
    let program = match construct(&contents) {
        Ok(v) => v,
        // TODO: Pass along error information
        Err(_) => return runtime_err(RuntimeErrorType::SoundChangeCompileError),
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

fn enforce_single<T>(input: Vec<T>) -> Result<T, GenerativeProgramRuntimeError>
where
    T: Clone,
{
    if input.len() == 1 {
        Ok(input[0].clone())
    } else {
        runtime_err(RuntimeErrorType::MismatchedRangeLengths)
    }
}
