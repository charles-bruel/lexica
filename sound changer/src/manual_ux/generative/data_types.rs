// The following code has been taken from Lagomorph
// git@github.com:BigTandy/Lagomorph with permission
// from the author and owner

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

// Modified for lexica
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Keyword {
    Foreach,
    Filter,
    Save,
    Saved,
    Enum,
    String,
    Int,
    UInt
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
    Pipe,
}