// The following code has been taken from Lagomorph
// git@github.com:BigTandy/Lagomorph with permission
// from the author

use super::tokenizer::Token;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PrimitiveDataTypes {
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NumericLiteralEncoding {
    Base, //Regular encoding
    Bin,  //Binary encoding,      prefixed with 0b
    Oct,  //Octal encoding,       prefixed with 0o
    Hex,  //Hexadecimal encoding, prefixed with 0x
    Exp,  //Exponential, i.e. 10E-6, only for base 10 encodings
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StringLiteralEncoding {
    Base, //Regular encoding
    Raw,  //No escaped characters, WYSIWG
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Keyword {
    Function,
    Let,
    Mut,
    Const,
    For,
    While,
    If,
    Else,
    Elif,
    Break,
    Continue,
    True,
    False,
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Define,
    Export,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Operator {
    Arrow,
    Plus,
    Minus,
    Star,
    Slash,
    Equals,
    PlusPlus,
    MinusMinus,
    StarStar,
    PlusEquals,
    MinusEquals,
    StarEquals,
    SlashEquals,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    Comma,
    SemiColon,
    Bang,
    Dollar,
}

pub struct CompilationUnit {
    //This is still really light on details and stuff
    pub tokens: Vec<Token>,
    pub source: String,
}

impl PrimitiveDataTypes {
    pub fn sizeof(&self) -> usize {
        match self {
            PrimitiveDataTypes::Bool => 1,
            PrimitiveDataTypes::I8   => 1,
            PrimitiveDataTypes::I16  => 2,
            PrimitiveDataTypes::I32  => 4,
            PrimitiveDataTypes::I64  => 8,
            PrimitiveDataTypes::U8   => 1,
            PrimitiveDataTypes::U16  => 2,
            PrimitiveDataTypes::U32  => 4,
            PrimitiveDataTypes::U64  => 8,
            PrimitiveDataTypes::F32  => 4,
            PrimitiveDataTypes::F64  => 8,
        }
    }
}