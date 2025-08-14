//! WASM bindings for Susumu language - browser execution
//! 
//! This module provides JavaScript bindings for running Susumu code in the browser
//! with full visual debugging and performance monitoring capabilities.

use crate::{Interpreter, Lexer, Parser};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// Import the `console.log` function from the browser
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    #[wasm_bindgen(js_namespace = console)]
    fn warn(s: &str);
    
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

// Custom console logging macros for WASM
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[allow(unused_macros)]
macro_rules! console_warn {
    ($($t:tt)*) => (warn(&format_args!($($t)*).to_string()))
}

macro_rules! console_error {
    ($($t:tt)*) => (error(&format_args!($($t)*).to_string()))
}

/// JavaScript-friendly execution result
#[derive(Serialize, Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub result: String,
    pub error: Option<String>,
    pub execution_time_ms: f64,
    pub debug_info: Option<DebugInfo>,
}

/// Visual debugging information for the browser
#[derive(Serialize, Deserialize, Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct DebugInfo {
    pub execution_traces: String,
    pub performance_stats: String,
    pub flow_diagram: String,
    pub arrow_flow_svg: Option<String>,
}

/// Performance statistics for JavaScript
#[derive(Serialize, Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct PerformanceStats {
    pub total_expressions: u64,
    pub execution_time_ns: u64,
    pub arrow_chains: u64,
    pub function_calls: u64,
    pub convergence_operations: u64,
}

/// Main WASM interface for Susumu execution
#[wasm_bindgen]
pub struct SusumuEngine {
    interpreter: Interpreter,
}

#[wasm_bindgen]
impl SusumuEngine {
    /// Create a new Susumu engine instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> SusumuEngine {
        // Set panic hook for better error messages in browser
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        
        console_log!("🚀 Susumu WASM Engine initialized");
        
        SusumuEngine {
            interpreter: Interpreter::new(),
        }
    }

    /// Execute Susumu code and return result
    #[wasm_bindgen]
    pub fn execute(&mut self, source: &str) -> JsValue {
        let start_time = js_sys::Date::now();
        
        match self.execute_internal(source) {
            Ok(result) => {
                let execution_time = js_sys::Date::now() - start_time;
                
                let exec_result = ExecutionResult {
                    success: true,
                    result: serde_json::to_string(&result).unwrap_or_else(|_| "null".to_string()),
                    error: None,
                    execution_time_ms: execution_time,
                    debug_info: None,
                };
                
                serde_wasm_bindgen::to_value(&exec_result).unwrap_or(JsValue::NULL)
            }
            Err(e) => {
                let execution_time = js_sys::Date::now() - start_time;
                
                let exec_result = ExecutionResult {
                    success: false,
                    result: "null".to_string(),
                    error: Some(e.to_string()),
                    execution_time_ms: execution_time,
                    debug_info: None,
                };
                
                console_error!("Susumu execution error: {}", e);
                serde_wasm_bindgen::to_value(&exec_result).unwrap_or(JsValue::NULL)
            }
        }
    }

    /// Execute with full debugging information
    #[wasm_bindgen]
    pub fn execute_with_debug(&mut self, source: &str) -> JsValue {
        let start_time = js_sys::Date::now();
        
        match self.execute_internal(source) {
            Ok(result) => {
                let execution_time = js_sys::Date::now() - start_time;
                let traces = self.interpreter.get_execution_traces();
                let stats = self.interpreter.get_performance_stats();
                let flow_diagram = self.interpreter.generate_execution_diagram();
                
                let debug_info = DebugInfo {
                    execution_traces: self.format_execution_traces(traces),
                    performance_stats: self.format_performance_stats(stats),
                    flow_diagram,
                    arrow_flow_svg: Some(self.generate_arrow_flow_svg(traces)),
                };
                
                let exec_result = ExecutionResult {
                    success: true,
                    result: serde_json::to_string(&result).unwrap_or_else(|_| "null".to_string()),
                    error: None,
                    execution_time_ms: execution_time,
                    debug_info: Some(debug_info),
                };
                
                console_log!("✅ Susumu execution completed with debug info");
                serde_wasm_bindgen::to_value(&exec_result).unwrap_or(JsValue::NULL)
            }
            Err(e) => {
                let execution_time = js_sys::Date::now() - start_time;
                
                let exec_result = ExecutionResult {
                    success: false,
                    result: "null".to_string(),
                    error: Some(e.to_string()),
                    execution_time_ms: execution_time,
                    debug_info: None,
                };
                
                console_error!("❌ Susumu execution error: {}", e);
                serde_wasm_bindgen::to_value(&exec_result).unwrap_or(JsValue::NULL)
            }
        }
    }

    /// Get built-in function documentation
    #[wasm_bindgen]
    pub fn get_builtin_functions(&self) -> JsValue {
        let builtins = vec![
            ("add", "Add numbers: 5 -> add <- 3 -> add <- 2"),
            ("multiply", "Multiply numbers: 5 -> multiply <- 3"),
            ("subtract", "Subtract numbers: 10 -> subtract <- 3"),
            ("divide", "Divide numbers: 15 -> divide <- 3"),
            ("concat", "Concatenate strings: \"hello\" -> concat <- \" world\""),
            ("length", "Get length: \"hello\" -> length or [1,2,3] -> length"),
            ("first", "Get first element: [1,2,3] -> first"),
            ("last", "Get last element: [1,2,3] -> last"),
            ("print", "Print values: 42 -> print"),
            ("to_upper", "Convert to uppercase: \"hello\" -> to_upper"),
            ("to_lower", "Convert to lowercase: \"HELLO\" -> to_lower"),
        ];
        
        serde_wasm_bindgen::to_value(&builtins).unwrap_or(JsValue::NULL)
    }

    /// Check syntax without executing
    #[wasm_bindgen]
    pub fn check_syntax(&self, source: &str) -> JsValue {
        match Lexer::new(source).tokenize() {
            Ok(tokens) => {
                match Parser::new(tokens).parse() {
                    Ok(_) => {
                        let result = serde_json::json!({
                            "valid": true,
                            "error": null
                        });
                        serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
                    }
                    Err(e) => {
                        let result = serde_json::json!({
                            "valid": false,
                            "error": e.to_string()
                        });
                        serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
                    }
                }
            }
            Err(e) => {
                let result = serde_json::json!({
                    "valid": false,
                    "error": e.to_string()
                });
                serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
            }
        }
    }

    /// Get version information
    #[wasm_bindgen]
    pub fn version(&self) -> String {
        "Susumu WASM v0.1.0 - High-performance arrow-flow programming".to_string()
    }
}

impl SusumuEngine {
    fn execute_internal(&mut self, source: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let tokens = Lexer::new(source).tokenize()?;
        let ast = Parser::new(tokens).parse()?;
        let result = self.interpreter.execute(&ast)?;
        Ok(result)
    }

    fn format_execution_traces(&self, traces: &[crate::interpreter::ExecutionTrace]) -> String {
        let mut output = String::new();
        output.push_str("Execution Traces:\n");
        output.push_str("================\n\n");
        
        for (i, trace) in traces.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", i + 1, trace.expression));
            output.push_str(&format!("   Time: {}μs\n", trace.execution_time_ns / 1000));
            output.push_str(&format!("   Result: {:?}\n\n", trace.output_value));
        }
        
        output
    }

    fn format_performance_stats(&self, stats: &crate::interpreter::PerformanceStats) -> String {
        format!(
            "Performance Statistics:\n\
            ======================\n\
            • Total expressions: {}\n\
            • Execution time: {}μs\n\
            • Arrow chains: {}\n\
            • Function calls: {}\n\
            • Convergence operations: {}",
            stats.total_expressions_evaluated,
            stats.total_execution_time_ns / 1000,
            stats.arrow_chain_count,
            stats.function_call_count,
            stats.convergence_operations
        )
    }

    fn generate_arrow_flow_svg(&self, traces: &[crate::interpreter::ExecutionTrace]) -> String {
        let mut svg = String::new();
        svg.push_str(r#"<svg width="800" height="400" xmlns="http://www.w3.org/2000/svg">"#);
        svg.push_str(r#"<style>
            .arrow-text { font-family: 'Courier New', monospace; font-size: 14px; fill: #333; }
            .arrow-line { stroke: #4CAF50; stroke-width: 2; fill: none; }
            .arrow-head { fill: #4CAF50; }
            .node { fill: #E3F2FD; stroke: #2196F3; stroke-width: 2; }
            .node-text { font-family: 'Arial', sans-serif; font-size: 12px; fill: #1976D2; text-anchor: middle; }
        </style>"#);
        
        let box_width = 120;
        let box_height = 40;
        let spacing = 150;
        let start_y = 50;
        
        for (i, trace) in traces.iter().take(5).enumerate() {
            let x = 50 + i * spacing;
            
            // Draw node box
            svg.push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" class="node" rx="5"/>"#,
                x, start_y, box_width, box_height
            ));
            
            // Draw text
            let text = if trace.expression.len() > 15 {
                format!("{}...", &trace.expression[..12])
            } else {
                trace.expression.clone()
            };
            
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" class="node-text">{}</text>"#,
                x + box_width / 2,
                start_y + box_height / 2 + 5,
                text
            ));
            
            // Draw arrow to next node
            if i < traces.len() - 1 && i < 4 {
                let arrow_start_x = x + box_width;
                let arrow_end_x = arrow_start_x + spacing - box_width;
                let arrow_y = start_y + box_height / 2;
                
                // Arrow line
                svg.push_str(&format!(
                    r#"<line x1="{}" y1="{}" x2="{}" y2="{}" class="arrow-line"/>"#,
                    arrow_start_x, arrow_y, arrow_end_x, arrow_y
                ));
                
                // Arrow head
                svg.push_str(&format!(
                    r#"<polygon points="{},{} {},{} {},{}" class="arrow-head"/>"#,
                    arrow_end_x, arrow_y,
                    arrow_end_x - 10, arrow_y - 5,
                    arrow_end_x - 10, arrow_y + 5
                ));
            }
        }
        
        svg.push_str("</svg>");
        svg
    }
}

/// Standalone functions for JavaScript integration
#[wasm_bindgen]
pub fn execute_susumu(source: &str) -> JsValue {
    let mut engine = SusumuEngine::new();
    engine.execute(source)
}

#[wasm_bindgen]
pub fn execute_susumu_with_debug(source: &str) -> JsValue {
    let mut engine = SusumuEngine::new();
    engine.execute_with_debug(source)
}

#[wasm_bindgen]
pub fn check_susumu_syntax(source: &str) -> JsValue {
    let engine = SusumuEngine::new();
    engine.check_syntax(source)
}

/// Initialize WASM module
#[wasm_bindgen(start)]
pub fn main() {
    console_log!("🌐 Susumu WASM module loaded successfully!");
    console_log!("📚 Use SusumuEngine.new() to create an interpreter instance");
}