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
4. Literals are placed into equations
5. To reference another table, use this format: TABLE_ID:COLUMN_NAME.
   You can omit one of the values if appropiate
6. The foreach function creates an entry for each row in the the table,
   with the contents given by the specified column
Example:
POS|word|translation
...|String|String
:={=foreach(1:POS)|=foreach(1:word)+ka|=foreach(1:translation)}

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
   Note that enums literals have their source specified with the : operator
9. A row selection can be saved with the save(selection, name) command, which can then
   be loaded with the saved(name, <new base column>) command. Note how the column used
   can change
   Generative procedures are evaluated left to right and thus a configuration saved in
   column 2 cannot be used in column 1
Example:
POS|word|translation
...|String|String
:={=foreach(1:POS).filter(:POS==POS:Noun).save(a)|=saved(a,:word)+ka|=saved(a,:translation)}
:={=foreach(1:POS).filter(:POS!=POS:Noun).save(a)|=saved(a,:word)|=saved(a,:translation)}

The above code creates an entry in the table for every entry in the previous
table (id=1), and appends "ka" to every noun

*/

mod construction;
mod data_types;
mod execution;
mod tokenizer;

use std::{collections::HashMap, num::ParseIntError, rc::Rc};

use self::execution::OutputNode;

use super::{
    project::Project,
    table::{TableDescriptor, TableLoadingError, TableRow},
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GenerativeLine {
    pub columns: Vec<GenerativeProgram>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub enum GenerativeProgramRuntimeError {
    MismatchedRangeLengths,
    TypeMismatch,
    OutOfOrderExecution,
    TableNotFound,
    EnumNotFound,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum GenerativeProgramCompileError {
    SyntaxError(SyntaxErrorType),
    TypeMismatch,
    IntParseError(ParseIntError),
    IntOutOfRange,
    OnlySpecifiedTable,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub enum SyntaxErrorType {
    InvalidKeywordDuringBlankStageParsing,
    InvalidTokenDuringBlankStageParsing,
    InvalidTokenDuringKeywordParsing,
    InvalidTokenDuringTableColumnSpecifierParsing(u32),
    MissingProgramSurrondings,
    ExpectedOpenParethesis,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GenerativeProgram {
    output_node: OutputNode,
}

pub fn parse_generative_table_line(
    descriptor: Rc<TableDescriptor>,
    line: &str,
) -> Result<TableRow, TableLoadingError> {
    construction::parse_generative_table_line(descriptor, line)
}
