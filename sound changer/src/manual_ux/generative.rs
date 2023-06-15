/*
The generative table contents system is designed to give all the tools
nessecary to regenerative a complex and evolving conlang from the basic
inputs.

The generative system is structured around generating the contents of each
column individually. Since the generative system must be able to generative
content both one line at a time, and entire blocks (for example one generative
line to evolve every word from the previous timeframe), the system must be able
to support a number of different configurations of quantities.

At a basic level, the system requires a consistent size among the various columns.
If column 1 generates 12 rows and column 2 generates 10 rows, the system will
crash. The exception, however, is if the a column generates 1 element - in that
case, the element will be duplicated for every row.
This is made more complex with various levels of mutations. A mutation is
another step one can add to a process and it will generate multiple outputs per
input. Mutations occur in levels and are specially marked so that not everything
has to include them. For exmaple, a 12 element list can have one column with a 3x
mutation on it, which would generate 36 elements, with the other columns being
duplicated 3 times.

Internally this is implemented as an AST for each column, with changes affecting
program state also being nodes. The AST is recursively executed when needed.

Syntax discussion:
All generative lines begin with the := operator and must be surronded in {}
After that operator, columns are seperated like normal, with the | symbol
1. Constant values are encoded as usual
2. Variables are marked by beginning the block with the = symbol
3. Values can be added (int, uint) or concatenated (string) with the + symbol
4. Literals are encoded by the lit() function in equations
5. To reference another table, use this format: TABLE_ID:COLUMN_NAME.
   You can omit one of the values if appropiate
6. The foreach function creates an entry for each row in the the table, 
   with the contents given by the specified column
Example:
POS|word|translation
...|String|String
:={=foreach(1:POS)|=foreach(1:word)+lit(ka)|=foreach(1:translation)}

The above code creates an entry in the table for every entry in the previous
table (id=1), and appends "ka" to the word

7. To run a word through a sound change program, use the sc(word, PROGRAM_NAME)
   function
   A function called (.fun()) will use the thing it is called on as the first 
   parameter
8. A selection can be filtered based on several conditionals using the 
   filter(items, condition) command. 
   This must be called directly after a foreach, filter, etc., because once it
   is used, it loses the multi-element properties.
   Note how the selection parameter can be different than the contents parameter
9. A row selection can be saved with the save(selection, name) command, which can then
   be loaded with the saved(name, <new base column>) command. Note how the column used
   can change
   Generative procedures are evaluated left to right and thus a configuration saved in
   column 2 cannot be used in column 1
Example:
POS|word|translation
...|String|String
:={=foreach(1:POS).filter(:POS==Noun).save(a)|=saved(a,:word)+lit(ka)|=saved(a,:translation)}
:={=foreach(1:POS).filter(:POS!=Noun).save(a)|=saved(a,:word)|=saved(a,:translation)}

The above code creates an entry in the table for every entry in the previous
table (id=1), and appends "ka" to every noun

*/

use std::collections::HashMap;

use super::table::TableRow;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GenerativeLine {
    pub columns: Vec<GenerativeProgram>,
}

pub enum GenerativeProgramRuntimeError {
    MismatchedRangeLengths,
    TypeMismatch,
    OutOfOrderExecution,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GenerativeProgram {
    output_node: OutputNode,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct ExecutionContext {
    pub saved_ranges: HashMap<String, Range>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum OutputNode {
    String(StringNode),
    Int(IntNode),
    UInt(UIntNode),
    Enum(EnumNode),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum StringNode {
    LiteralNode(String),
    AdditionNode(Box<StringNode>, Box<StringNode>),
    ConversionNode(RangeNode)
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum IntNode {
    LiteralNode(i32),
    AdditionNode(Box<IntNode>, Box<IntNode>),
    ConversionNode(RangeNode)
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum UIntNode {
    LiteralNode(u32),
    AdditionNode(Box<UIntNode>, Box<UIntNode>),
    ConversionNode(RangeNode)
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum EnumNode {
    LiteralNode,
    ConversionNode(RangeNode)
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum RangeNode {
    ForeachNode(TableSpecifier, Option<ColumnSpecifier>),
    FilterNode(Box<RangeNode>, ColumnSpecifier, Box<FilterPredicate>),
    Save(Box<RangeNode>, String),
    Saved(String, Option<ColumnSpecifier>)
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct Range {
    pub rows: Vec<TableRow>,
    pub column_id: Option<usize>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct TableSpecifier {
    pub table_id: usize,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct ColumnSpecifier {
    pub column_id: usize,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum FilterPredicate {
    EnumCompare(SimpleComparisionType, EnumNode),
    StringCompare(SimpleComparisionType, StringNode),
    IntCompare(ComplexComparisionType, IntNode),
    UIntCompare(ComplexComparisionType, UIntNode),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum SimpleComparisionType {
    Equals, NotEquals
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum ComplexComparisionType {
    Equals, NotEquals, Greater, Less, GreaterEquals, LessEquals
}

impl StringNode {
    pub fn eval(&self, context: &mut ExecutionContext) -> Result<Vec<String>, GenerativeProgramRuntimeError> {
        match self {
            StringNode::LiteralNode(contents) => Ok(vec!(contents.clone())),
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
            },
            StringNode::ConversionNode(_) => todo!(),
        }
    }
}

impl IntNode {
    pub fn eval(&self, context: &mut ExecutionContext) -> Result<Vec<i32>, GenerativeProgramRuntimeError> {
        match self {
            IntNode::LiteralNode(contents) => Ok(vec!(*contents)),
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
            },
            IntNode::ConversionNode(_) => todo!(),
        }
    }
}

impl UIntNode {
    pub fn eval(&self, context: &mut ExecutionContext) -> Result<Vec<u32>, GenerativeProgramRuntimeError> {
        match self {
            UIntNode::LiteralNode(contents) => Ok(vec!(*contents)),
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
            },
            UIntNode::ConversionNode(_) => todo!(),
        }
    }
}

impl RangeNode {
    pub fn eval(self, context: &mut ExecutionContext) -> Result<Range, GenerativeProgramRuntimeError> {
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
            },
            RangeNode::Save(range, key) => {
                let evaluated = range.to_owned().eval(context)?;
                context.saved_ranges.insert(key, evaluated.clone());
                return Ok(evaluated);
            },
            RangeNode::Saved(key, column) => {
                let mut result = context.saved_ranges[&key].clone();
                match column {
                    Some(v) => result.column_id = Some(v.column_id),
                    None => (),
                }

                return Ok(result);
            },
        }
    }
}

impl FilterPredicate {
    pub fn check(&self, row: &TableRow, column: &ColumnSpecifier, context: &mut ExecutionContext) -> Result<bool, GenerativeProgramRuntimeError> {
        // Check data type of input
        let (descriptor, contents) = match row {
            TableRow::PopulatedTableRow { descriptor, contents, .. } => (descriptor, contents),
            TableRow::UnpopulatedTableRow { descriptor, .. } => {
                return Err(GenerativeProgramRuntimeError::OutOfOrderExecution);
            },
        };
        let input_data_type = &descriptor.column_descriptors[column.column_id].data_type;
        
        // Crazy multi-level match statement
        match self {
            FilterPredicate::EnumCompare(comp_type, node) => match input_data_type {
                super::table::TableDataTypeDescriptor::Enum(_) => match contents[column.column_id] {
                    super::table::TableContents::Enum(v) => todo!(),
                    // Assuming that the creation of the descriptor is done correctly,
                    // this will never happen and will be unreachable
                    _ => unreachable!(),
                },
                _ => return Err(GenerativeProgramRuntimeError::MismatchedRangeLengths),
            },
            FilterPredicate::StringCompare(comp_type, node) => match input_data_type {
                super::table::TableDataTypeDescriptor::String => match &contents[column.column_id] {
                    super::table::TableContents::String(v) => {
                        let eval = enforce_single_string(node.eval(context)?)?;
                        return match comp_type {
                            SimpleComparisionType::Equals => Ok(eval == *v),
                            SimpleComparisionType::NotEquals => Ok(eval != *v),
                        };
                    },
                    // Assuming that the creation of the descriptor is done correctly,
                    // this will never happen and will be unreachable
                    _ => unreachable!(),
                },
                _ => return Err(GenerativeProgramRuntimeError::MismatchedRangeLengths),
            },
            FilterPredicate::IntCompare(comp_type, node) => match input_data_type {
                super::table::TableDataTypeDescriptor::Int => match contents[column.column_id] {
                    super::table::TableContents::Int(v) => todo!(),
                    // Assuming that the creation of the descriptor is done correctly,
                    // this will never happen and will be unreachable
                    _ => unreachable!(),
                },
                _ => return Err(GenerativeProgramRuntimeError::MismatchedRangeLengths),
            },
            FilterPredicate::UIntCompare(comp_type, node) => match input_data_type {
                super::table::TableDataTypeDescriptor::UInt => match contents[column.column_id] {
                    super::table::TableContents::UInt(v) => todo!(),
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