//! Python FFI bridge for Susumu using PyO3
//!
//! Provides a Python API for executing Susumu code with full debugging capabilities.

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use serde_json::Value;
// use std::collections::HashMap;

use crate::error::SusumuError;
use crate::interpreter::{ExecutionTrace, PerformanceStats};
use crate::{Interpreter, Lexer, Parser};

/// Python exception for Susumu errors
#[derive(Debug)]
struct SusumuPythonError(SusumuError);

impl From<SusumuError> for PyErr {
    fn from(err: SusumuError) -> PyErr {
        PyRuntimeError::new_err(err.to_string())
    }
}

/// Convert serde_json::Value to Python object
fn json_to_python(py: Python, value: &Value) -> PyResult<PyObject> {
    match value {
        Value::Null => Ok(py.None()),
        Value::Bool(b) => Ok(b.to_object(py)),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.to_object(py))
            } else if let Some(f) = n.as_f64() {
                Ok(f.to_object(py))
            } else {
                Ok(py.None())
            }
        }
        Value::String(s) => Ok(s.to_object(py)),
        Value::Array(arr) => {
            let py_list = PyList::empty(py);
            for item in arr {
                py_list.append(json_to_python(py, item)?)?;
            }
            Ok(py_list.to_object(py))
        }
        Value::Object(obj) => {
            let py_dict = PyDict::new(py);
            for (key, value) in obj {
                py_dict.set_item(key, json_to_python(py, value)?)?;
            }
            Ok(py_dict.to_object(py))
        }
    }
}

/// Convert Python object to serde_json::Value (for future use)
fn python_to_json(py: Python, obj: &PyAny) -> PyResult<Value> {
    if obj.is_none() {
        Ok(Value::Null)
    } else if let Ok(b) = obj.extract::<bool>() {
        Ok(Value::Bool(b))
    } else if let Ok(i) = obj.extract::<i64>() {
        Ok(serde_json::json!(i))
    } else if let Ok(f) = obj.extract::<f64>() {
        Ok(serde_json::json!(f))
    } else if let Ok(s) = obj.extract::<String>() {
        Ok(Value::String(s))
    } else if let Ok(list) = obj.downcast::<PyList>() {
        let mut arr = Vec::new();
        for item in list {
            arr.push(python_to_json(py, item)?);
        }
        Ok(Value::Array(arr))
    } else if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut obj = serde_json::Map::new();
        for (key, value) in dict {
            let key_str = key.extract::<String>()?;
            obj.insert(key_str, python_to_json(py, value)?);
        }
        Ok(Value::Object(obj))
    } else {
        // Fallback: convert to string
        Ok(Value::String(obj.to_string()))
    }
}

/// Execution result for Python API
#[pyclass]
#[derive(Clone)]
pub struct ExecutionResult {
    #[pyo3(get)]
    pub success: bool,
    #[pyo3(get)]
    pub result: PyObject,
    #[pyo3(get)]
    pub error: Option<String>,
    #[pyo3(get)]
    pub execution_time_ms: f64,
}

/// Debug information for Python API
#[pyclass]
#[derive(Clone)]
pub struct DebugInfo {
    #[pyo3(get)]
    pub execution_traces: Vec<String>,
    #[pyo3(get)]
    pub performance_stats: PyObject,
    #[pyo3(get)]
    pub flow_diagram: String,
}

/// Main Susumu engine for Python
#[pyclass]
pub struct SusumuEngine {
    interpreter: Interpreter,
}

#[pymethods]
impl SusumuEngine {
    #[new]
    fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
        }
    }

    /// Get the version of the Susumu engine
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// Execute Susumu code and return result
    fn execute(&mut self, py: Python, source: &str) -> PyResult<ExecutionResult> {
        let start_time = std::time::Instant::now();

        match self.execute_internal(source) {
            Ok(result) => {
                let execution_time = start_time.elapsed().as_secs_f64() * 1000.0;

                Ok(ExecutionResult {
                    success: true,
                    result: json_to_python(py, &result)?,
                    error: None,
                    execution_time_ms: execution_time,
                })
            }
            Err(e) => {
                let execution_time = start_time.elapsed().as_secs_f64() * 1000.0;

                Ok(ExecutionResult {
                    success: false,
                    result: py.None(),
                    error: Some(e.to_string()),
                    execution_time_ms: execution_time,
                })
            }
        }
    }

    /// Execute with full debugging information
    fn execute_with_debug(
        &mut self,
        py: Python,
        source: &str,
    ) -> PyResult<(ExecutionResult, Option<DebugInfo>)> {
        let start_time = std::time::Instant::now();

        match self.execute_internal(source) {
            Ok(result) => {
                let execution_time = start_time.elapsed().as_secs_f64() * 1000.0;
                let traces = self.interpreter.get_execution_traces();
                let stats = self.interpreter.get_performance_stats();
                let flow_diagram = self.interpreter.generate_execution_diagram();

                let debug_info = DebugInfo {
                    execution_traces: traces.iter().map(|t| self.format_trace(t)).collect(),
                    performance_stats: self.format_performance_stats(py, stats)?,
                    flow_diagram,
                };

                let exec_result = ExecutionResult {
                    success: true,
                    result: json_to_python(py, &result)?,
                    error: None,
                    execution_time_ms: execution_time,
                };

                Ok((exec_result, Some(debug_info)))
            }
            Err(e) => {
                let execution_time = start_time.elapsed().as_secs_f64() * 1000.0;

                let exec_result = ExecutionResult {
                    success: false,
                    result: py.None(),
                    error: Some(e.to_string()),
                    execution_time_ms: execution_time,
                };

                Ok((exec_result, None))
            }
        }
    }

    /// Check syntax without executing
    fn check_syntax(&self, source: &str) -> PyResult<(bool, Option<String>)> {
        match Lexer::new(source).tokenize() {
            Ok(tokens) => match Parser::new(tokens).parse() {
                Ok(_) => Ok((true, None)),
                Err(e) => Ok((false, Some(e.to_string()))),
            },
            Err(e) => Ok((false, Some(e.to_string()))),
        }
    }

    /// Reset the interpreter state
    fn reset(&mut self) {
        self.interpreter = Interpreter::new();
    }
}

impl SusumuEngine {
    fn execute_internal(&mut self, source: &str) -> Result<Value, SusumuError> {
        let tokens = Lexer::new(source).tokenize()?;
        let ast = Parser::new(tokens).parse()?;
        self.interpreter.execute(&ast)
    }

    fn format_trace(&self, trace: &ExecutionTrace) -> String {
        format!(
            "{}: {} -> {}",
            trace.expression,
            self.value_to_string(&trace.input_value),
            self.value_to_string(&trace.output_value)
        )
    }

    fn format_performance_stats(&self, py: Python, stats: &PerformanceStats) -> PyResult<PyObject> {
        let dict = PyDict::new(py);
        dict.set_item("total_expressions", stats.total_expressions_evaluated)?;
        dict.set_item("execution_time_ns", stats.total_execution_time_ns)?;
        dict.set_item("arrow_chains", stats.arrow_chain_count)?;
        dict.set_item("function_calls", stats.function_call_count)?;
        dict.set_item("convergence_operations", stats.convergence_operations)?;
        dict.set_item("parallel_operations", stats.parallel_operations)?;
        Ok(dict.to_object(py))
    }

    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            Value::Array(_) => "[array]".to_string(),
            Value::Object(_) => "{object}".to_string(),
        }
    }
}

/// Simple function API for quick execution
#[pyfunction]
fn execute(py: Python, source: &str) -> PyResult<PyObject> {
    let mut engine = SusumuEngine::new();
    let result = engine.execute(py, source)?;

    if result.success {
        Ok(result.result)
    } else {
        Err(PyRuntimeError::new_err(
            result.error.unwrap_or_else(|| "Unknown error".to_string()),
        ))
    }
}

/// Simple syntax check function
#[pyfunction]
fn check_syntax(source: &str) -> PyResult<bool> {
    let engine = SusumuEngine::new();
    let (valid, _) = engine.check_syntax(source)?;
    Ok(valid)
}

/// Python module definition
#[pymodule]
fn susumu(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(execute, m)?)?;
    m.add_function(wrap_pyfunction!(check_syntax, m)?)?;
    m.add_class::<SusumuEngine>()?;
    m.add_class::<ExecutionResult>()?;
    m.add_class::<DebugInfo>()?;

    // Add version info
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__doc__", "Susumu arrow-flow programming language")?;

    Ok(())
}
