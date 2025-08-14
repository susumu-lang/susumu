//! Susumu Programming Language - Rust Implementation
//!
//! A high-performance arrow-flow programming language that makes data transformations
//! visually explicit through arrow syntax.

pub mod ast;
pub mod builtins;
pub mod environment;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;
// External tests module removed - using inline tests instead
pub mod types;
pub mod visual_debug;

// #[cfg(feature = "parallel")]
// pub mod parallel;

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(feature = "python-bridge")]
pub mod python;

#[cfg(feature = "lsp")]
pub mod lsp;

pub use ast::{Expression, Program, Statement};
pub use error::{SusumuError, SusumuResult};
pub use interpreter::Interpreter;
pub use lexer::{Lexer, Token};
pub use parser::Parser;

/// Main entry point for executing Susumu code
pub fn execute(source: &str) -> SusumuResult<serde_json::Value> {
    let tokens = Lexer::new(source).tokenize()?;
    let ast = Parser::new(tokens).parse()?;
    let mut interpreter = Interpreter::new();
    interpreter.execute(&ast)
}

/// Execute Susumu code and return the result as a string
pub fn execute_to_string(source: &str) -> String {
    match execute(source) {
        Ok(value) => {
            // Don't output null results to match Python behavior
            if value == serde_json::Value::Null {
                String::new()
            } else {
                serde_json::to_string(&value).unwrap_or_else(|_| "null".to_string())
            }
        }
        Err(err) => format!("Error: {}", err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arrow_chain() {
        let result = execute("5 -> add <- 3 -> multiply <- 2");
        assert!(result.is_ok());
    }

    #[test]
    fn test_success_error_pattern() {
        let code = r#"
        testFunction(x) {
            x -> validate -> i success {
                result -> return
            } e {
                error -> error <- "validation failed"
            }
        }
        "#;

        let result = execute(code);
        assert!(result.is_ok());
    }
}
