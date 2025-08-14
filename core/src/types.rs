//! Type system for Susumu with compile-time safety and inference

// use crate::error::{SusumuError, SusumuResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Core type system for Susumu with arrow-flow awareness
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SusumuType {
    // Primitive types
    Number,
    String,
    Boolean,
    Null,

    // Collection types
    Array(Box<SusumuType>),
    Tuple(Vec<SusumuType>),
    Object(Vec<(String, SusumuType)>),

    // Function types with arrow-flow semantics
    Function {
        params: Vec<SusumuType>,
        return_type: Box<SusumuType>,
        /// Indicates if this function can handle convergent arrows
        supports_convergence: bool,
    },

    // Arrow chain types
    ArrowChain {
        input_type: Box<SusumuType>,
        output_type: Box<SusumuType>,
        /// Types at each step in the chain for debugging
        intermediate_types: Vec<SusumuType>,
    },

    // Success/Error result types
    Result {
        success_type: Box<SusumuType>,
        error_type: Box<SusumuType>,
    },

    // Generic types for inference
    Generic(String),

    // Unknown type (for inference)
    Unknown,

    // Union types for flexible typing
    Union(Vec<SusumuType>),
}

impl SusumuType {
    /// Check if this type can be assigned to another type
    pub fn is_assignable_to(&self, other: &SusumuType) -> bool {
        match (self, other) {
            // Exact matches
            (a, b) if a == b => true,

            // Unknown can be assigned to anything
            (SusumuType::Unknown, _) | (_, SusumuType::Unknown) => true,

            // Union type compatibility
            (SusumuType::Union(types), target) => types.iter().all(|t| t.is_assignable_to(target)),
            (source, SusumuType::Union(types)) => types.iter().any(|t| source.is_assignable_to(t)),

            // Array covariance
            (SusumuType::Array(a), SusumuType::Array(b)) => a.is_assignable_to(b),

            // Tuple covariance
            (SusumuType::Tuple(a), SusumuType::Tuple(b)) => {
                a.len() == b.len()
                    && a.iter()
                        .zip(b.iter())
                        .all(|(ta, tb)| ta.is_assignable_to(tb))
            }

            // Function contravariance for parameters, covariance for return
            (
                SusumuType::Function {
                    params: p1,
                    return_type: r1,
                    ..
                },
                SusumuType::Function {
                    params: p2,
                    return_type: r2,
                    ..
                },
            ) => {
                p1.len() == p2.len()
                    && p2
                        .iter()
                        .zip(p1.iter())
                        .all(|(pa, pb)| pa.is_assignable_to(pb))
                    && r1.is_assignable_to(r2)
            }

            // Result type covariance
            (
                SusumuType::Result {
                    success_type: s1,
                    error_type: e1,
                },
                SusumuType::Result {
                    success_type: s2,
                    error_type: e2,
                },
            ) => s1.is_assignable_to(s2) && e1.is_assignable_to(e2),

            _ => false,
        }
    }

    /// Get a human-readable description of this type
    pub fn description(&self) -> String {
        match self {
            SusumuType::Number => "number".to_string(),
            SusumuType::String => "string".to_string(),
            SusumuType::Boolean => "boolean".to_string(),
            SusumuType::Null => "null".to_string(),
            SusumuType::Array(inner) => format!("array of {}", inner.description()),
            SusumuType::Tuple(types) => {
                let type_names: Vec<String> = types.iter().map(|t| t.description()).collect();
                format!("tuple ({})", type_names.join(", "))
            }
            SusumuType::Object(fields) => {
                if fields.is_empty() {
                    "object".to_string()
                } else {
                    let field_types: Vec<String> = fields
                        .iter()
                        .map(|(k, v)| format!("{}: {}", k, v.description()))
                        .collect();
                    format!("object {{ {} }}", field_types.join(", "))
                }
            }
            SusumuType::Function {
                params,
                return_type,
                supports_convergence,
            } => {
                let param_types: Vec<String> = params.iter().map(|t| t.description()).collect();
                let convergence = if *supports_convergence {
                    " (supports convergence)"
                } else {
                    ""
                };
                format!(
                    "function ({}) -> {}{}",
                    param_types.join(", "),
                    return_type.description(),
                    convergence
                )
            }
            SusumuType::ArrowChain {
                input_type,
                output_type,
                ..
            } => {
                format!(
                    "arrow chain {} -> {}",
                    input_type.description(),
                    output_type.description()
                )
            }
            SusumuType::Result {
                success_type,
                error_type,
            } => {
                format!(
                    "result<{}, {}>",
                    success_type.description(),
                    error_type.description()
                )
            }
            SusumuType::Generic(name) => format!("generic {}", name),
            SusumuType::Unknown => "unknown".to_string(),
            SusumuType::Union(types) => {
                let type_names: Vec<String> = types.iter().map(|t| t.description()).collect();
                format!("union ({})", type_names.join(" | "))
            }
        }
    }

    /// Check if this is a result type
    pub fn is_result(&self) -> bool {
        matches!(self, SusumuType::Result { .. })
    }

    /// Get the success type if this is a result type
    pub fn success_type(&self) -> Option<&SusumuType> {
        match self {
            SusumuType::Result { success_type, .. } => Some(success_type),
            _ => None,
        }
    }

    /// Get the error type if this is a result type
    pub fn error_type(&self) -> Option<&SusumuType> {
        match self {
            SusumuType::Result { error_type, .. } => Some(error_type),
            _ => None,
        }
    }

    /// Create a result type
    pub fn result(success_type: SusumuType, error_type: SusumuType) -> Self {
        SusumuType::Result {
            success_type: Box::new(success_type),
            error_type: Box::new(error_type),
        }
    }

    /// Create a function type
    pub fn function(
        params: Vec<SusumuType>,
        return_type: SusumuType,
        supports_convergence: bool,
    ) -> Self {
        SusumuType::Function {
            params,
            return_type: Box::new(return_type),
            supports_convergence,
        }
    }

    /// Create an arrow chain type
    pub fn arrow_chain(
        input_type: SusumuType,
        output_type: SusumuType,
        intermediate_types: Vec<SusumuType>,
    ) -> Self {
        SusumuType::ArrowChain {
            input_type: Box::new(input_type),
            output_type: Box::new(output_type),
            intermediate_types,
        }
    }
}

impl fmt::Display for SusumuType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Type environment for tracking variable and function types
#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    variables: HashMap<String, SusumuType>,
    functions: Vec<(String, SusumuType)>,
    parent: Option<Box<TypeEnvironment>>,
}

impl TypeEnvironment {
    /// Create a new empty type environment
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: Vec::new(),
            parent: None,
        }
    }

    /// Create a type environment with a parent
    pub fn with_parent(parent: TypeEnvironment) -> Self {
        Self {
            variables: HashMap::new(),
            functions: Vec::new(),
            parent: Some(Box::new(parent)),
        }
    }

    /// Define a variable type
    pub fn define_variable(&mut self, name: String, var_type: SusumuType) {
        self.variables.insert(name, var_type);
    }

    /// Define a function type
    pub fn define_function(&mut self, name: String, func_type: SusumuType) {
        self.functions.push((name, func_type));
    }

    /// Get a variable type
    pub fn get_variable(&self, name: &str) -> Option<&SusumuType> {
        self.variables
            .get(name)
            .or_else(|| self.parent.as_ref().and_then(|p| p.get_variable(name)))
    }

    /// Get a function type
    pub fn get_function(&self, name: &str) -> Option<&SusumuType> {
        self.functions
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, t)| t)
            .or_else(|| self.parent.as_ref().and_then(|p| p.get_function(name)))
    }
}

impl Default for TypeEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

/// Type checker for compile-time type safety
pub struct TypeChecker {
    pub env: TypeEnvironment,
    pub errors: Vec<TypeError>,
}

/// Rich type error with suggestions for fixes
#[derive(Debug, Clone)]
pub struct TypeError {
    pub line: usize,
    pub column: usize,
    pub error_type: TypeErrorKind,
    pub suggestion: String,
}

#[derive(Debug, Clone)]
pub enum TypeErrorKind {
    TypeMismatch {
        expected: SusumuType,
        found: SusumuType,
        context: String,
    },
    ArrowChainError {
        step: usize,
        expected_input: SusumuType,
        actual_input: SusumuType,
        function_name: String,
    },
    ConvergenceError {
        function_name: String,
        expected_types: Vec<SusumuType>,
        actual_types: Vec<SusumuType>,
    },
    UndefinedVariable {
        name: String,
        similar_names: Vec<String>,
    },
    UndefinedFunction {
        name: String,
        similar_names: Vec<String>,
    },
    ResultTypeError {
        context: String,
        expected_result: bool,
        actual_type: SusumuType,
    },
}

impl TypeChecker {
    /// Create a new type checker with built-in types
    pub fn new() -> Self {
        let mut env = TypeEnvironment::new();
        Self::setup_builtin_types(&mut env);

        Self {
            env,
            errors: Vec::new(),
        }
    }

    /// Setup built-in function types
    fn setup_builtin_types(env: &mut TypeEnvironment) {
        // Math functions with convergence support
        env.define_function(
            "add".to_string(),
            SusumuType::function(
                vec![SusumuType::Number, SusumuType::Number],
                SusumuType::Number,
                true, // Supports convergence: 5 -> add <- 3 <- 2
            ),
        );

        env.define_function(
            "multiply".to_string(),
            SusumuType::function(
                vec![SusumuType::Number, SusumuType::Number],
                SusumuType::Number,
                true,
            ),
        );

        env.define_function(
            "subtract".to_string(),
            SusumuType::function(
                vec![SusumuType::Number, SusumuType::Number],
                SusumuType::Number,
                false, // No convergence: order matters
            ),
        );

        // String functions
        env.define_function(
            "concat".to_string(),
            SusumuType::function(
                vec![SusumuType::String, SusumuType::String],
                SusumuType::String,
                true,
            ),
        );

        env.define_function(
            "length".to_string(),
            SusumuType::function(
                vec![SusumuType::Union(vec![
                    SusumuType::String,
                    SusumuType::Array(Box::new(SusumuType::Unknown)),
                ])],
                SusumuType::Number,
                false,
            ),
        );

        // I/O functions
        env.define_function(
            "print".to_string(),
            SusumuType::function(
                vec![SusumuType::Unknown],
                SusumuType::Unknown,
                true, // Can print multiple values: a -> print <- b <- c
            ),
        );

        // Array functions
        env.define_function(
            "first".to_string(),
            SusumuType::function(
                vec![SusumuType::Array(Box::new(SusumuType::Generic(
                    "T".to_string(),
                )))],
                SusumuType::Union(vec![SusumuType::Generic("T".to_string()), SusumuType::Null]),
                false,
            ),
        );

        env.define_function(
            "push".to_string(),
            SusumuType::function(
                vec![
                    SusumuType::Array(Box::new(SusumuType::Generic("T".to_string()))),
                    SusumuType::Generic("T".to_string()),
                ],
                SusumuType::Array(Box::new(SusumuType::Generic("T".to_string()))),
                false,
            ),
        );
    }

    /// Generate a helpful error message with fix suggestions
    pub fn generate_error_message(&self, error: &TypeError) -> String {
        let mut message = String::new();

        message.push_str(&format!(
            "Type Error at line {}, column {}:\n",
            error.line, error.column
        ));

        match &error.error_type {
            TypeErrorKind::TypeMismatch {
                expected,
                found,
                context,
            } => {
                message.push_str(&format!("  Expected type: {}\n", expected.description()));
                message.push_str(&format!("  Found type:    {}\n", found.description()));
                message.push_str(&format!("  Context:       {}\n", context));

                if let (SusumuType::Number, SusumuType::String) = (expected, found) {
                    message.push_str(
                        "  💡 Suggestion: Use 'to_number()' to convert string to number\n",
                    );
                } else if let (SusumuType::String, SusumuType::Number) = (expected, found) {
                    message.push_str(
                        "  💡 Suggestion: Use 'to_string()' to convert number to string\n",
                    );
                }
            }

            TypeErrorKind::ArrowChainError {
                step,
                expected_input,
                actual_input,
                function_name,
            } => {
                message.push_str(&format!("  Arrow chain type error at step {}\n", step));
                message.push_str(&format!(
                    "  Function '{}' expects: {}\n",
                    function_name,
                    expected_input.description()
                ));
                message.push_str(&format!(
                    "  But receives:          {}\n",
                    actual_input.description()
                ));
                message.push_str("  💡 Visual Debug: The arrow flow shows the type mismatch:\n");
                message.push_str(&format!(
                    "     {} -> {} <- [type mismatch here]\n",
                    actual_input.description(),
                    function_name
                ));
            }

            TypeErrorKind::ConvergenceError {
                function_name,
                expected_types,
                actual_types,
            } => {
                message.push_str(&format!(
                    "  Convergence error in function '{}'\n",
                    function_name
                ));
                message.push_str("  Expected converging types: ");
                for (i, t) in expected_types.iter().enumerate() {
                    if i > 0 {
                        message.push_str(", ");
                    }
                    message.push_str(&t.description());
                }
                message.push('\n');
                message.push_str("  Actual converging types:   ");
                for (i, t) in actual_types.iter().enumerate() {
                    if i > 0 {
                        message.push_str(", ");
                    }
                    message.push_str(&t.description());
                }
                message.push('\n');
                message.push_str("  💡 Visual Debug: Check your convergent arrows:\n");
                message.push_str(&format!("     a -> {} <- b <- c\n", function_name));
                message.push_str("     ^    ^      ^    ^\n");
                message.push_str("     Types must be compatible for convergence\n");
            }

            TypeErrorKind::UndefinedVariable {
                name,
                similar_names,
            } => {
                message.push_str(&format!("  Undefined variable: '{}'\n", name));
                if !similar_names.is_empty() {
                    message.push_str("  💡 Did you mean: ");
                    message.push_str(&similar_names.join(", "));
                    message.push('\n');
                }
            }

            TypeErrorKind::UndefinedFunction {
                name,
                similar_names,
            } => {
                message.push_str(&format!("  Undefined function: '{}'\n", name));
                if !similar_names.is_empty() {
                    message.push_str("  💡 Did you mean: ");
                    message.push_str(&similar_names.join(", "));
                    message.push('\n');
                }
                message
                    .push_str("  💡 Available functions: add, multiply, subtract, print, length\n");
            }

            TypeErrorKind::ResultTypeError {
                context,
                expected_result,
                actual_type,
            } => {
                message.push_str(&format!("  Result type error in {}\n", context));
                if *expected_result {
                    message.push_str("  Expected: result type for 'i success { ... } e { ... }'\n");
                    message.push_str(&format!("  Found:    {}\n", actual_type.description()));
                    message.push_str("  💡 Suggestion: The expression before 'i success' should return a result type\n");
                    message.push_str("     Functions that can fail should use 'result<success_type, error_type>'\n");
                } else {
                    message.push_str("  Expected: non-result type\n");
                    message.push_str(&format!("  Found:    {}\n", actual_type.description()));
                }
            }
        }

        message.push_str(&format!("\n  💡 {}\n", error.suggestion));
        message
    }

    /// Find similar names for typo suggestions
    #[allow(dead_code)]
    fn find_similar_names(&self, target: &str, names: &[String]) -> Vec<String> {
        names
            .iter()
            .filter(|name| self.levenshtein_distance(target, name) <= 2)
            .take(3)
            .cloned()
            .collect()
    }

    /// Simple Levenshtein distance for typo detection
    #[allow(dead_code)]
    fn levenshtein_distance(&self, a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let a_len = a_chars.len();
        let b_len = b_chars.len();

        let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

        for i in 0..=a_len {
            matrix[i][0] = i;
        }
        for j in 0..=b_len {
            matrix[0][j] = j;
        }

        for i in 1..=a_len {
            for j in 1..=b_len {
                let cost = if a_chars[i - 1] == b_chars[j - 1] {
                    0
                } else {
                    1
                };
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                    matrix[i - 1][j - 1] + cost,
                );
            }
        }

        matrix[a_len][b_len]
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_assignability() {
        let number = SusumuType::Number;
        let string = SusumuType::String;
        let unknown = SusumuType::Unknown;

        assert!(number.is_assignable_to(&number));
        assert!(!number.is_assignable_to(&string));
        assert!(number.is_assignable_to(&unknown));
        assert!(unknown.is_assignable_to(&number));
    }

    #[test]
    fn test_function_type_compatibility() {
        let func1 = SusumuType::function(vec![SusumuType::Number], SusumuType::String, false);

        let func2 = SusumuType::function(vec![SusumuType::Number], SusumuType::String, false);

        assert!(func1.is_assignable_to(&func2));
    }

    #[test]
    fn test_result_type_creation() {
        let result_type =
            SusumuType::result(SusumuType::String, SusumuType::Object(HashMap::new()));

        assert!(result_type.is_result());
        assert_eq!(result_type.success_type(), Some(&SusumuType::String));
    }
}
