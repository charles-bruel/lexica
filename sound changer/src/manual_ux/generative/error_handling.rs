// The following code has been taken from Lagomorph
// git@github.com:BigTandy/Lagomorph with permission
// from the author and owner

use std::vec;

use super::tokenizer::Token;

/// Stores possible state after returning from error handling. This
/// would have been named Result if not for name collisions with
/// std::Result
pub enum Out<T, E> {
    Good(T),
    Warn(T, Vec<E>),
    Err(Option<T>, Vec<E>),
}

impl<T, E> Out<T, E> {
    /// Returns a Out::Good if warnings is empty, if not returns Out::Warn
    pub fn new(val: T, warnings: Vec<E>) -> Out<T, E> {
        if warnings.len() == 0 {
            Out::Good(val)
        } else {
            Out::Warn(val, warnings)
        }
    }

    /// Returns Out::Good
    pub fn success(val: T) -> Out<T, E> {
        Out::Good(val)
    }

    /// Returns Out::Err
    pub fn err(val: Option<T>, errors: Vec<E>) -> Out<T, E> {
        Out::Err(val, errors)
    }

    /// Returns Out::Err. Use this only if the err is impossible to recover from,
    /// otherwise attempt to move on and catch as many errors as possible
    pub fn fatal(val: Option<T>, error: E) -> Out<T, E> {
        Out::Err(val, vec![error])
    }

    pub fn unwrap(self) -> T {
        match self {
            Out::Good(v) => v,
            Out::Warn(v, _) => v,
            Out::Err(Some(v), _) => v,
            Out::Err(None, _) => panic!("called `Out::unwrap()` on a `Err::None` value"),
        }
    }

    pub fn unwrap_err(self) -> Vec<E> {
        match self {
            Out::Good(_) => panic!("called `Out::unwrap_err()` on a `Good` value"),
            Out::Warn(_, _) => panic!("called `Out::unwrap_err()` on a `Warn` value"),
            Out::Err(_, v) => v,
        }
    }

    pub fn unwrap_warn(self) -> Vec<E> {
        match self {
            Out::Good(_) => panic!("called `Out::unwrap_warn()` on a `Good` value"),
            Out::Warn(_, v) => v,
            Out::Err(_, _) => panic!("called `Out::unwrap_warn()` on a `Warn` value"),
        }
    }

    pub fn unwrap_err_warn(self) -> Vec<E> {
        match self {
            Out::Good(_) => panic!("called `Out::unwrap_err_warn()` on a `Good` value"),
            Out::Warn(_, v) => v,
            Out::Err(_, v) => v,
        }
    }
}

pub struct OutBuilder {
    warnings: Vec<CompilationError>,
    errors: Vec<CompilationError>,
    recoverable: bool
}

impl OutBuilder {
    pub fn new() -> OutBuilder {
        OutBuilder {
            warnings: Vec::new(),
            errors: Vec::new(),
            recoverable: true,
        }
    }

    pub fn err(&mut self, error: CompilationError) {
        if cfg!(debug_assertions) {
            assert_eq!(error.error_type.get_severity(), Severity::Err);
        }
        self.errors.push(error);
    }
    
    pub fn fatal(&mut self, error: CompilationError) {
        if cfg!(debug_assertions) {
            assert_eq!(error.error_type.get_severity(), Severity::Err);
        }
        self.errors.push(error);
        self.recoverable = false;
    }

    pub fn warn(&mut self, warning: CompilationError) {
        if cfg!(debug_assertions) {
            assert_eq!(warning.error_type.get_severity(), Severity::Warn);
        }
        self.warnings.push(warning);
    }

    pub fn out<T>(val: T, builder: OutBuilder) -> Out<T, CompilationError> {
        if builder.errors.len() != 0 {
            if builder.recoverable {
                Out::err(Some(val), builder.errors)
            } else {
                Out::err(None, builder.errors)
            }
        } else {
            Out::new(val, builder.warnings)
        }
    }

    pub fn test<T>(&mut self, input: Out<T, CompilationError>) -> Option<T> {
        match input {
            Out::Good(a) => Some(a),
            Out::Warn(a, mut b) => { self.warnings.append(&mut b); Some(a) },
            Out::Err(a, mut b) => { self.errors.append(&mut b); a },
        }
    }

    pub fn test_fatal<T>(&mut self, input: Out<T, CompilationError>) -> Option<T> {
        match input {
            Out::Good(a) => Some(a),
            Out::Warn(a, mut b) => { self.warnings.append(&mut b); self.recoverable = false; Some(a) },
            Out::Err(a, mut b) => { self.errors.append(&mut b); a },
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CompilationError {
    pub error_type: CompilationErrorType,
    //TODO : Redo error message formatting
    pub message: &'static str,
    pub attribution: Token,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CompilationErrorType {
    MalformedHead,
    MalformedBody,
    UnknownParameter,
    IncompleteMacro,
    MacroNameCollision,
    MalformedMacroUsage,
    UnknownMacro,
    MissingParameter,
    TooManyParameters,
    InfiniteRecursion,
    UnterminatedBlockComment,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Severity {
    Warn, Err
}

impl CompilationErrorType {
    pub fn get_severity(&self) -> Severity {
        match self {
            CompilationErrorType::MalformedHead            |
            CompilationErrorType::MalformedBody            |
            CompilationErrorType::UnknownParameter         |
            CompilationErrorType::IncompleteMacro          |
            CompilationErrorType::MacroNameCollision       |
            CompilationErrorType::MalformedMacroUsage      |
            CompilationErrorType::UnknownMacro             |
            CompilationErrorType::MissingParameter         |
            CompilationErrorType::TooManyParameters        |
            CompilationErrorType::InfiniteRecursion        |
            CompilationErrorType::UnterminatedBlockComment => Severity::Err,
        }
    }
}

impl CompilationError {
    pub fn get_severity(&self) -> Severity {
        self.error_type.get_severity()
    }
}