// The following code has been taken from Lagomorph
// git@github.com:BigTandy/Lagomorph with permission
// from the author and owner

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
    SoundChange,
    Enum,
    String,
    Int,
    UInt,
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
    Equality,
    Inequality,
    Period,
    Comma,
    Colon,
    SemiColon,
    Bang,
    Dollar,
    Pipe,
}
