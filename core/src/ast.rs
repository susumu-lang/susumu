//! Abstract Syntax Tree definitions for Susumu language

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub functions: Vec<FunctionDef>,
    pub main_expression: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    Expression(Expression),
    FunctionDef(FunctionDef),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<String>,
    pub body: Expression,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    // Literals
    Number(f64),
    String(String),
    Boolean(bool),
    Null,

    // Identifiers
    Identifier(String),

    // Collections
    Tuple(Vec<Expression>),
    Object(Vec<(String, Expression)>),
    Array(Vec<Expression>),

    // Core arrow-flow constructs
    ArrowChain {
        expressions: Vec<Expression>,
        directions: Vec<ArrowDirection>,
    },

    // Function calls
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },

    // Control flow
    Conditional {
        condition_type: ConditionType,
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_if_branches: Vec<ElseIfBranch>,
        else_branch: Option<Box<Expression>>,
    },

    // Flow control statements
    Return(Box<Expression>),
    Error(Box<Expression>),

    // Iteration
    ForEach {
        variable: String,
        iterable: Box<Expression>,
        body: Box<Expression>,
    },

    // Blocks
    Block(Vec<Expression>),

    // Pattern matching
    Match {
        expr: Option<Box<Expression>>, // None for arrow chain integration
        cases: Vec<MatchCase>,
    },

    // Maybe and Result types
    Maybe {
        value: Option<Box<Expression>>, // None for 'none', Some(expr) for 'some(expr)'
    },
    Result {
        is_success: bool,
        value: Box<Expression>,
    },

    // Assignment
    Assignment {
        target: String,
        value: Box<Expression>,
        mutable: bool,
    },

    // Property access
    PropertyAccess {
        object: Box<Expression>,
        property: String,
    },

    // Binary operations
    BinaryOp {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },

    // Annotations
    Annotated {
        annotation: Annotation,
        expression: Box<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchCase {
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub body: Expression,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElseIfBranch {
    pub condition_type: ConditionType,
    pub condition: Expression,
    pub then_branch: Expression,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    Literal(LiteralValue),
    Identifier(String),
    Wildcard,
    Tuple(Vec<Pattern>),
    Object(Vec<(String, Pattern)>),
    ArrowPattern {
        constructor: String, // "some", "none", "success", "error"
        arg: Box<Pattern>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LiteralValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOperator {
    Add,       // +
    Subtract,  // -
    Multiply,  // *
    Divide,    // /
    Equal,     // ==
    NotEqual,  // !=
    Less,      // <
    Greater,   // >
    LessEq,    // <=
    GreaterEq, // >=
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArrowDirection {
    Forward,  // ->
    Backward, // <-
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConditionType {
    If,             // Traditional if condition
    Success,        // i success { ... } e { ... }
    Custom(String), // i customCondition { ... } e { ... }
}

impl Program {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            main_expression: None,
        }
    }

    pub fn add_function(&mut self, func: FunctionDef) {
        self.functions.push(func);
    }

    pub fn set_main_expression(&mut self, expr: Expression) {
        self.main_expression = Some(expr);
    }
}

impl Expression {
    /// Create a simple arrow chain with forward direction
    pub fn arrow_chain(expressions: Vec<Expression>) -> Self {
        let directions = vec![ArrowDirection::Forward; expressions.len().saturating_sub(1)];
        Self::ArrowChain {
            expressions,
            directions,
        }
    }

    /// Create an arrow chain with mixed directions for convergence
    pub fn convergence_chain(
        expressions: Vec<Expression>,
        directions: Vec<ArrowDirection>,
    ) -> Self {
        Self::ArrowChain {
            expressions,
            directions,
        }
    }

    /// Create a success/error conditional
    pub fn success_conditional(
        condition: Expression,
        then_branch: Expression,
        else_branch: Option<Expression>,
    ) -> Self {
        Self::Conditional {
            condition_type: ConditionType::Success,
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_if_branches: Vec::new(),
            else_branch: else_branch.map(Box::new),
        }
    }

    /// Create a custom conditional (e.g., i customCheck { ... } e { ... })
    pub fn custom_conditional(
        condition_name: String,
        condition: Expression,
        then_branch: Expression,
        else_branch: Option<Expression>,
    ) -> Self {
        Self::Conditional {
            condition_type: ConditionType::Custom(condition_name),
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_if_branches: Vec::new(),
            else_branch: else_branch.map(Box::new),
        }
    }

    /// Check if this expression is a literal value
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            Expression::Number(_)
                | Expression::String(_)
                | Expression::Boolean(_)
                | Expression::Null
        )
    }

    /// Check if this expression is an identifier
    pub fn is_identifier(&self) -> bool {
        matches!(self, Expression::Identifier(_))
    }

    /// Extract identifier name if this is an identifier
    pub fn as_identifier(&self) -> Option<&str> {
        match self {
            Expression::Identifier(name) => Some(name),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Annotation {
    Trace(String),             // @trace <- "payment-flow"
    Monitor(Vec<String>),      // @monitor <- ["latency", "errors"]
    Config(serde_json::Value), // @config <- {trace: "payment-flow", timeout: "30s"}
    Parallel,                  // @parallel
    Debug(Option<String>),     // @debug or @debug <- "checkpoint"
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}
