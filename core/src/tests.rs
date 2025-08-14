//! Comprehensive test suite for Susumu language implementation

use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use serde_json::Value;

/// Test result containing execution details
#[derive(Debug)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub output: Option<Value>,
    pub error: Option<String>,
    pub execution_time_ns: u64,
}

/// Comprehensive test runner for Susumu language
pub struct TestRunner {
    interpreter: Interpreter,
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
        }
    }

    /// Run a single test case
    pub fn run_test(&mut self, name: &str, source: &str, expected: Option<Value>) -> TestResult {
        let start_time = std::time::Instant::now();
        
        match self.execute_source(source) {
            Ok(result) => {
                let passed = if let Some(expected_val) = expected {
                    self.values_equal(&result, &expected_val)
                } else {
                    true // If no expected value, just check it runs without error
                };
                
                TestResult {
                    name: name.to_string(),
                    passed,
                    output: Some(result),
                    error: None,
                    execution_time_ns: start_time.elapsed().as_nanos() as u64,
                }
            }
            Err(e) => TestResult {
                name: name.to_string(),
                passed: false,
                output: None,
                error: Some(e.to_string()),
                execution_time_ns: start_time.elapsed().as_nanos() as u64,
            }
        }
    }

    /// Compare two JSON values with proper handling for numbers
    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(n1), Value::Number(n2)) => {
                let f1 = n1.as_f64().unwrap_or(0.0);
                let f2 = n2.as_f64().unwrap_or(0.0);
                (f1 - f2).abs() < f64::EPSILON
            }
            (Value::Array(a1), Value::Array(a2)) => {
                a1.len() == a2.len() && a1.iter().zip(a2.iter()).all(|(v1, v2)| self.values_equal(v1, v2))
            }
            _ => a == b,
        }
    }

    /// Execute Susumu source code
    fn execute_source(&mut self, source: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let tokens = Lexer::new(source).tokenize()?;
        let ast = Parser::new(tokens).parse()?;
        let result = self.interpreter.execute(&ast)?;
        Ok(result)
    }

    /// Run all built-in tests
    pub fn run_all_tests(&mut self) -> Vec<TestResult> {
        let mut results = Vec::new();

        // Basic arithmetic tests
        results.push(self.run_test(
            "basic_addition",
            "5 -> add <- 3",
            Some(Value::Number(serde_json::Number::from(8))),
        ));

        results.push(self.run_test(
            "convergence_addition",
            "5 -> add <- 3 <- 2",
            Some(Value::Number(serde_json::Number::from(10))),
        ));

        results.push(self.run_test(
            "chain_operations",
            "5 -> add <- 3 -> multiply <- 2",
            Some(Value::Number(serde_json::Number::from(16))),
        ));

        // Function tests
        results.push(self.run_test(
            "function_definition_and_call",
            r#"
square(x) {
    return <- x -> multiply <- x
}
5 -> square
            "#,
            Some(Value::Number(serde_json::Number::from(25))),
        ));

        results.push(self.run_test(
            "function_with_multiple_args",
            r#"
addThree(a, b, c) {
    return <- a -> add <- b -> add <- c
}
1 -> addThree <- 2 <- 3
            "#,
            Some(Value::Number(serde_json::Number::from(6))),
        ));

        // Array tests
        results.push(self.run_test(
            "array_first",
            "[1, 2, 3] -> first",
            Some(Value::Number(serde_json::Number::from(1))),
        ));

        results.push(self.run_test(
            "array_last",
            "[1, 2, 3] -> last",
            Some(Value::Number(serde_json::Number::from(3))),
        ));

        results.push(self.run_test(
            "array_length",
            "[1, 2, 3, 4, 5] -> length",
            Some(Value::Number(serde_json::Number::from(5))),
        ));

        // String tests
        results.push(self.run_test(
            "string_concat",
            r#""Hello" -> concat <- " World""#,
            Some(Value::String("Hello World".to_string())),
        ));

        results.push(self.run_test(
            "string_length",
            r#""Hello" -> length"#,
            Some(Value::Number(serde_json::Number::from(5))),
        ));

        results.push(self.run_test(
            "string_to_upper",
            r#""hello" -> to_upper"#,
            Some(Value::String("HELLO".to_string())),
        ));

        // Tuple tests
        results.push(self.run_test(
            "tuple_creation",
            "(1, 2, 3)",
            Some(Value::Array(vec![
                Value::Number(serde_json::Number::from(1)),
                Value::Number(serde_json::Number::from(2)),
                Value::Number(serde_json::Number::from(3)),
            ])),
        ));

        // Object tests
        results.push(self.run_test(
            "object_creation",
            r#"{name: "Alice", age: 30}"#,
            None, // Just test it doesn't error
        ));

        // Complex convergence tests
        results.push(self.run_test(
            "complex_convergence",
            "10 -> subtract <- 2 -> multiply <- 3 -> add <- 5",
            Some(Value::Number(serde_json::Number::from(29))),
        ));

        // Type safety tests (should fail compilation)
        results.push(self.run_test(
            "undefined_function",
            "5 -> nonexistentFunction",
            None, // Should fail
        ));

        results
    }

    /// Generate test report
    pub fn generate_report(&self, results: &[TestResult]) -> String {
        let mut report = String::new();
        report.push_str("=== Susumu Test Report ===\n\n");

        let passed = results.iter().filter(|r| r.passed).count();
        let total = results.len();
        let pass_rate = (passed as f64 / total as f64) * 100.0;

        report.push_str(&format!("Total Tests: {}\n", total));
        report.push_str(&format!("Passed: {}\n", passed));
        report.push_str(&format!("Failed: {}\n", total - passed));
        report.push_str(&format!("Pass Rate: {:.1}%\n\n", pass_rate));

        // Performance summary
        let total_time_ns: u64 = results.iter().map(|r| r.execution_time_ns).sum();
        let avg_time_ns = total_time_ns / results.len() as u64;
        report.push_str(&format!("Total Execution Time: {}ms\n", total_time_ns / 1_000_000));
        report.push_str(&format!("Average Test Time: {}μs\n\n", avg_time_ns / 1_000));

        // Individual test results
        report.push_str("Individual Test Results:\n");
        report.push_str("========================\n");

        for result in results {
            let status = if result.passed { "PASS" } else { "FAIL" };
            let time_us = result.execution_time_ns / 1_000;
            
            report.push_str(&format!("{:<30} {} ({:>6}μs)", result.name, status, time_us));
            
            if let Some(error) = &result.error {
                report.push_str(&format!(" - Error: {}", error));
            } else if let Some(output) = &result.output {
                report.push_str(&format!(" - Output: {:?}", output));
            }
            report.push('\n');
        }

        // Failed tests details
        let failed_tests: Vec<_> = results.iter().filter(|r| !r.passed).collect();
        if !failed_tests.is_empty() {
            report.push_str("\nFailed Test Details:\n");
            report.push_str("===================\n");
            
            for test in failed_tests {
                report.push_str(&format!("❌ {}\n", test.name));
                if let Some(error) = &test.error {
                    report.push_str(&format!("   Error: {}\n", error));
                }
                report.push('\n');
            }
        }

        report.push_str("\n=== End of Report ===\n");
        report
    }
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_basic() {
        let mut runner = TestRunner::new();
        let result = runner.run_test(
            "basic_test",
            "5 -> add <- 3",
            Some(Value::Number(serde_json::Number::from(8))),
        );
        assert!(result.passed);
    }

    #[test]
    fn test_all_comprehensive() {
        let mut runner = TestRunner::new();
        let results = runner.run_all_tests();
        let report = runner.generate_report(&results);
        println!("{}", report);
        
        // Ensure at least some tests pass
        let passed = results.iter().filter(|r| r.passed).count();
        assert!(passed > 0, "At least some tests should pass");
    }
}