//! Error handling for Susumu language

use std::fmt;
use thiserror::Error;

pub type SusumuResult<T> = Result<T, SusumuError>;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum SusumuError {
    #[error("Lexer error at line {line}, column {column}: {message}")]
    LexerError {
        line: usize,
        column: usize,
        message: String,
    },

    #[error("Parser error at line {line}: {message}")]
    ParserError {
        line: usize,
        message: String,
    },

    #[error("Runtime error: {message}")]
    RuntimeError {
        message: String,
    },

    #[error("Type error: expected {expected}, found {found}")]
    TypeError {
        expected: String,
        found: String,
    },

    #[error("Undefined variable: {name}")]
    UndefinedVariable {
        name: String,
    },

    #[error("Undefined function: {name}")]
    UndefinedFunction {
        name: String,
    },

    #[error("Function call error: {message}")]
    FunctionCallError {
        message: String,
    },

    #[error("Arrow chain error: {message}")]
    ArrowChainError {
        message: String,
    },

    #[error("User-defined error: {value:?}")]
    UserError {
        value: serde_json::Value,
    },

    #[error("Return value: {value:?}")]
    ReturnValue {
        value: serde_json::Value,
    },

    #[error("IO error: {message}")]
    IoError {
        message: String,
    },
}

impl SusumuError {
    pub fn lexer_error(line: usize, column: usize, message: impl Into<String>) -> Self {
        Self::LexerError {
            line,
            column,
            message: message.into(),
        }
    }

    pub fn parser_error(line: usize, message: impl Into<String>) -> Self {
        Self::ParserError {
            line,
            message: message.into(),
        }
    }

    pub fn runtime_error(message: impl Into<String>) -> Self {
        Self::RuntimeError {
            message: message.into(),
        }
    }

    pub fn type_error(expected: impl Into<String>, found: impl Into<String>) -> Self {
        Self::TypeError {
            expected: expected.into(),
            found: found.into(),
        }
    }

    pub fn undefined_variable(name: impl Into<String>) -> Self {
        Self::UndefinedVariable {
            name: name.into(),
        }
    }

    pub fn undefined_function(name: impl Into<String>) -> Self {
        Self::UndefinedFunction {
            name: name.into(),
        }
    }

    pub fn function_call_error(message: impl Into<String>) -> Self {
        Self::FunctionCallError {
            message: message.into(),
        }
    }

    pub fn arrow_chain_error(message: impl Into<String>) -> Self {
        Self::ArrowChainError {
            message: message.into(),
        }
    }

    pub fn user_error(value: serde_json::Value) -> Self {
        Self::UserError { value }
    }

    pub fn return_value(value: serde_json::Value) -> Self {
        Self::ReturnValue { value }
    }

    pub fn io_error(message: impl Into<String>) -> Self {
        Self::IoError {
            message: message.into(),
        }
    }
}

// Custom flow control errors for return and user-defined errors
#[derive(Debug, Clone, PartialEq)]
pub enum FlowControl {
    Return(serde_json::Value),
    Error(serde_json::Value),
}

impl fmt::Display for FlowControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlowControl::Return(value) => write!(f, "Return: {:?}", value),
            FlowControl::Error(value) => write!(f, "Error: {:?}", value),
        }
    }
}

impl std::error::Error for FlowControl {}