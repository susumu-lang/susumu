//! Visual debugging tools for Susumu arrow-flow programs

use crate::interpreter::{ExecutionStepType, ExecutionTrace};
use crate::parser::ArrowFlowPath;
// use crate::types::SusumuType;
// use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Visual debugging session that tracks execution flow
#[derive(Debug, Clone)]
pub struct VisualDebugger {
    /// All execution traces from the current session
    traces: Vec<ExecutionTrace>,
    /// Arrow flow paths with type information
    flow_paths: Vec<ArrowFlowPath>,
    /// Breakpoints set by the user
    breakpoints: Vec<Breakpoint>,
    /// Current debugging state
    #[allow(dead_code)]
    state: DebugState,
}

/// Debugging state information
#[derive(Debug, Clone)]
pub struct DebugState {
    pub current_line: usize,
    pub current_column: usize,
    pub call_stack: Vec<StackFrame>,
    pub variables: HashMap<String, Value>,
    pub is_paused: bool,
    pub step_mode: StepMode,
}

/// Call stack frame for debugging
#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function_name: String,
    pub line: usize,
    pub column: usize,
    pub local_variables: HashMap<String, Value>,
}

/// Breakpoint configuration
#[derive(Debug, Clone)]
pub struct Breakpoint {
    pub line: usize,
    pub column: Option<usize>,
    pub condition: Option<String>,
    pub enabled: bool,
    pub hit_count: usize,
}

/// Step debugging modes
#[derive(Debug, Clone, PartialEq)]
pub enum StepMode {
    None,
    StepInto,
    StepOver,
    StepOut,
    Continue,
}

/// Visual flow diagram generator
pub struct FlowDiagramGenerator {
    /// Configuration for diagram generation
    config: DiagramConfig,
}

#[derive(Debug, Clone)]
pub struct DiagramConfig {
    pub show_types: bool,
    pub show_values: bool,
    pub show_timing: bool,
    pub compact_mode: bool,
    pub color_coding: bool,
}

impl VisualDebugger {
    /// Create a new visual debugger
    pub fn new() -> Self {
        Self {
            traces: Vec::new(),
            flow_paths: Vec::new(),
            breakpoints: Vec::new(),
            state: DebugState {
                current_line: 0,
                current_column: 0,
                call_stack: Vec::new(),
                variables: HashMap::new(),
                is_paused: false,
                step_mode: StepMode::None,
            },
        }
    }

    /// Add execution trace for visual debugging
    pub fn add_trace(&mut self, trace: ExecutionTrace) {
        self.traces.push(trace);
    }

    /// Add arrow flow path for type visualization
    pub fn add_flow_path(&mut self, path: ArrowFlowPath) {
        self.flow_paths.push(path);
    }

    /// Set a breakpoint at the specified location
    pub fn set_breakpoint(
        &mut self,
        line: usize,
        column: Option<usize>,
        condition: Option<String>,
    ) -> usize {
        let breakpoint = Breakpoint {
            line,
            column,
            condition,
            enabled: true,
            hit_count: 0,
        };

        self.breakpoints.push(breakpoint);
        self.breakpoints.len() - 1 // Return breakpoint ID
    }

    /// Remove a breakpoint
    pub fn remove_breakpoint(&mut self, line: usize) {
        self.breakpoints.retain(|bp| bp.line != line);
    }

    /// Check if we should break at the current location
    pub fn should_break(&mut self, line: usize, column: usize) -> bool {
        for breakpoint in &mut self.breakpoints {
            if breakpoint.enabled && breakpoint.line == line {
                if let Some(bp_column) = breakpoint.column {
                    if bp_column != column {
                        continue;
                    }
                }

                breakpoint.hit_count += 1;

                // TODO: Evaluate condition if present
                return true;
            }
        }
        false
    }

    /// Generate comprehensive execution report
    pub fn generate_execution_report(&self) -> String {
        let mut report = String::new();

        report.push_str("SUSUMU VISUAL DEBUGGING REPORT\n");
        report.push_str("==============================\n\n");

        // Execution Summary
        report.push_str("EXECUTION SUMMARY:\n");
        report.push_str(&format!("  Total steps: {}\n", self.traces.len()));
        report.push_str(&format!("  Arrow flow paths: {}\n", self.flow_paths.len()));
        report.push_str(&format!("  Breakpoints: {}\n", self.breakpoints.len()));
        report.push('\n');

        // Arrow Flow Visualization
        report.push_str("ARROW FLOW VISUALIZATION:\n");
        report.push_str(&"-".repeat(40));
        report.push('\n');

        for (i, path) in self.flow_paths.iter().enumerate() {
            report.push_str(&format!(
                "Flow Path {} (line {}):\n",
                i + 1,
                path.start_line
            ));

            // Show the actual arrow flow
            for (j, step) in path.steps.iter().enumerate() {
                if j == 0 {
                    report.push_str("  ");
                    report.push_str(&step.expression);
                } else {
                    let arrow = match step.direction {
                        crate::ast::ArrowDirection::Forward => " -> ",
                        crate::ast::ArrowDirection::Backward => " <- ",
                    };
                    report.push_str(arrow);
                    report.push_str(&step.expression);
                }
            }
            report.push('\n');

            // Show type flow
            report.push_str("  Type Flow:\n");
            for (j, step) in path.steps.iter().enumerate() {
                report.push_str(&format!(
                    "    Step {}: {} -> {}\n",
                    j + 1,
                    step.input_type.description(),
                    step.output_type.description()
                ));
            }
            report.push('\n');
        }

        // Execution Trace
        report.push_str("EXECUTION TRACE:\n");
        report.push_str(&"-".repeat(40));
        report.push('\n');

        for (i, trace) in self.traces.iter().enumerate() {
            report.push_str(&format!(
                "{}. {} (line {})\n",
                i + 1,
                trace.expression,
                trace.line
            ));

            match &trace.step_type {
                ExecutionStepType::ArrowForward { from, to } => {
                    report.push_str(&format!("   {} -> {}\n", from, to));
                    report.push_str(&format!(
                        "   Input:  {}\n",
                        self.value_to_display(&trace.input_value)
                    ));
                    report.push_str(&format!(
                        "   Output: {}\n",
                        self.value_to_display(&trace.output_value)
                    ));
                }
                ExecutionStepType::ArrowBackward { from, to } => {
                    report.push_str(&format!("   {} <- {}\n", to, from));
                    report.push_str(&format!(
                        "   Convergent: {}\n",
                        self.value_to_display(&trace.input_value)
                    ));
                }
                ExecutionStepType::FunctionCall { name, args } => {
                    report.push_str(&format!("   Function: {}(", name));
                    for (j, arg) in args.iter().enumerate() {
                        if j > 0 {
                            report.push_str(", ");
                        }
                        report.push_str(&self.value_to_display(arg));
                    }
                    report.push_str(")\n");
                    report.push_str(&format!(
                        "   Result: {}\n",
                        self.value_to_display(&trace.output_value)
                    ));
                }
                ExecutionStepType::Conditional {
                    branch,
                    condition_result,
                } => {
                    report.push_str(&format!(
                        "   Branch: {} (condition: {})\n",
                        branch, condition_result
                    ));
                    report.push_str(&format!(
                        "   Result: {}\n",
                        self.value_to_display(&trace.output_value)
                    ));
                }
                _ => {
                    report.push_str(&format!(
                        "   Result: {}\n",
                        self.value_to_display(&trace.output_value)
                    ));
                }
            }

            if trace.execution_time_ns > 0 {
                report.push_str(&format!("   Time: {}ns\n", trace.execution_time_ns));
            }
            report.push('\n');
        }

        // Performance Analysis
        self.add_performance_analysis(&mut report);

        // Debugging Recommendations
        self.add_debugging_recommendations(&mut report);

        report
    }

    /// Generate interactive HTML debugging interface
    pub fn generate_html_interface(&self) -> String {
        let mut html = String::new();

        html.push_str(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>Susumu Visual Debugger</title>
    <style>
        body { font-family: 'Monaco', 'Menlo', monospace; margin: 0; padding: 20px; }
        .debugger { display: flex; height: 90vh; }
        .code-panel { flex: 1; border: 1px solid #ccc; margin-right: 10px; }
        .debug-panel { flex: 1; border: 1px solid #ccc; }
        .arrow-flow { background: #f8f9fa; padding: 10px; margin: 5px 0; border-radius: 4px; }
        .arrow { color: #007acc; font-weight: bold; }
        .type-info { color: #666; font-size: 0.9em; }
        .execution-step { border-left: 3px solid #28a745; padding-left: 10px; margin: 5px 0; }
        .error-step { border-left: 3px solid #dc3545; }
        .breakpoint { background: #fff3cd; border-left: 3px solid #ffc107; }
        .controls { margin-bottom: 10px; }
        .controls button { margin-right: 5px; padding: 5px 10px; }
    </style>
</head>
<body>
    <h1>🏹 Susumu Visual Debugger</h1>
    
    <div class="controls">
        <button onclick="stepInto()">Step Into</button>
        <button onclick="stepOver()">Step Over</button>
        <button onclick="continue_()">Continue</button>
        <button onclick="reset()">Reset</button>
    </div>
    
    <div class="debugger">
        <div class="code-panel">
            <h3>Arrow Flow Visualization</h3>
            <div id="arrow-flows">
        "#,
        );

        // Add arrow flow visualizations
        for (i, path) in self.flow_paths.iter().enumerate() {
            html.push_str(&format!(
                r#"
                <div class="arrow-flow">
                    <strong>Flow {} (line {})</strong><br>
                    <div style="font-family: monospace; font-size: 14px;">
            "#,
                i + 1,
                path.start_line
            ));

            // Generate visual arrow flow
            for (j, step) in path.steps.iter().enumerate() {
                if j == 0 {
                    html.push_str(&step.expression);
                } else {
                    let arrow = match step.direction {
                        crate::ast::ArrowDirection::Forward => " <span class=\"arrow\">→</span> ",
                        crate::ast::ArrowDirection::Backward => " <span class=\"arrow\">←</span> ",
                    };
                    html.push_str(arrow);
                    html.push_str(&step.expression);
                }
            }

            html.push_str("</div>");

            // Add type information
            html.push_str("<div class=\"type-info\">");
            for (j, step) in path.steps.iter().enumerate() {
                html.push_str(&format!(
                    "Step {}: {} → {}<br>",
                    j + 1,
                    step.input_type.description(),
                    step.output_type.description()
                ));
            }
            html.push_str("</div></div>");
        }

        html.push_str(
            r#"
            </div>
        </div>
        
        <div class="debug-panel">
            <h3>Execution Trace</h3>
            <div id="execution-trace">
        "#,
        );

        // Add execution traces
        for (i, trace) in self.traces.iter().enumerate() {
            html.push_str(&format!(
                r#"
                <div class="execution-step">
                    <strong>Step {}</strong>: {} (line {})<br>
                    Input: {}<br>
                    Output: {}<br>
            "#,
                i + 1,
                trace.expression,
                trace.line,
                self.value_to_display(&trace.input_value),
                self.value_to_display(&trace.output_value)
            ));

            if trace.execution_time_ns > 0 {
                html.push_str(&format!("Time: {}ns<br>", trace.execution_time_ns));
            }

            html.push_str("</div>");
        }

        html.push_str(
            r#"
            </div>
        </div>
    </div>
    
    <script>
        function stepInto() {
            console.log('Step Into');
            // Would communicate with debugger backend
        }
        
        function stepOver() {
            console.log('Step Over');
        }
        
        function continue_() {
            console.log('Continue');
        }
        
        function reset() {
            console.log('Reset');
        }
        
        // Auto-scroll to current execution point
        function scrollToCurrentStep() {
            const currentStep = document.querySelector('.execution-step:last-child');
            if (currentStep) {
                currentStep.scrollIntoView({ behavior: 'smooth' });
            }
        }
        
        scrollToCurrentStep();
    </script>
</body>
</html>
        "#,
        );

        html
    }

    fn add_performance_analysis(&self, report: &mut String) {
        report.push_str("PERFORMANCE ANALYSIS:\n");
        report.push_str(&"-".repeat(40));
        report.push('\n');

        let total_time: u64 = self.traces.iter().map(|t| t.execution_time_ns).sum();
        let avg_time = if !self.traces.is_empty() {
            total_time / self.traces.len() as u64
        } else {
            0
        };

        report.push_str(&format!(
            "  Total execution time: {}ms\n",
            total_time / 1_000_000
        ));
        report.push_str(&format!("  Average step time: {}ns\n", avg_time));

        // Find slowest operations
        let mut sorted_traces = self.traces.clone();
        sorted_traces.sort_by(|a, b| b.execution_time_ns.cmp(&a.execution_time_ns));

        report.push_str("  Slowest operations:\n");
        for (i, trace) in sorted_traces.iter().take(5).enumerate() {
            report.push_str(&format!(
                "    {}. {} ({}ns)\n",
                i + 1,
                trace.expression,
                trace.execution_time_ns
            ));
        }
        report.push('\n');
    }

    fn add_debugging_recommendations(&self, report: &mut String) {
        report.push_str("DEBUGGING RECOMMENDATIONS:\n");
        report.push_str(&"-".repeat(40));
        report.push('\n');

        // Analyze common patterns and suggest improvements
        let arrow_chains = self
            .traces
            .iter()
            .filter(|t| matches!(t.step_type, ExecutionStepType::ArrowForward { .. }))
            .count();

        let function_calls = self
            .traces
            .iter()
            .filter(|t| matches!(t.step_type, ExecutionStepType::FunctionCall { .. }))
            .count();

        if arrow_chains > function_calls * 2 {
            report.push_str("  💡 You have many arrow chains. Consider using more direct function calls for better performance.\n");
        }

        if self.flow_paths.iter().any(|p| p.steps.len() > 10) {
            report.push_str("  💡 Some arrow chains are very long. Consider breaking them into smaller, named functions.\n");
        }

        if self.traces.iter().any(|t| t.execution_time_ns > 1_000_000) {
            report.push_str("  💡 Some operations are slow. Use the performance analysis to identify bottlenecks.\n");
        }

        report.push_str(
            "  💡 Use 'i success {} e {}' patterns for better error flow visualization.\n",
        );
        report
            .push_str("  💡 Set breakpoints at arrow junctions to inspect intermediate values.\n");
        report.push_str("  💡 The HTML interface provides interactive debugging capabilities.\n");
        report.push('\n');
    }

    fn value_to_display(&self, value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => format!("\"{}\"", s),
            Value::Array(a) => format!("[{} items]", a.len()),
            Value::Object(o) => format!("{{object: {} fields}}", o.len()),
        }
    }
}

impl FlowDiagramGenerator {
    pub fn new() -> Self {
        Self {
            config: DiagramConfig {
                show_types: true,
                show_values: true,
                show_timing: false,
                compact_mode: false,
                color_coding: true,
            },
        }
    }

    /// Generate ASCII art flow diagram
    pub fn generate_ascii_diagram(&self, path: &ArrowFlowPath) -> String {
        let mut diagram = String::new();

        diagram.push_str("┌");
        diagram.push_str(&"─".repeat(50));
        diagram.push_str("┐\n");
        diagram.push_str(&format!(
            "│ Arrow Flow Diagram (line {:<29} │\n",
            path.start_line
        ));
        diagram.push_str("├");
        diagram.push_str(&"─".repeat(50));
        diagram.push_str("┤\n");

        // Generate flow visualization
        let mut current_line = String::new();
        for (i, step) in path.steps.iter().enumerate() {
            if i == 0 {
                current_line.push_str(&format!(" {} ", step.expression));
            } else {
                let arrow = match step.direction {
                    crate::ast::ArrowDirection::Forward => "──→",
                    crate::ast::ArrowDirection::Backward => "←──",
                };
                current_line.push_str(&format!(" {} {} ", arrow, step.expression));
            }
        }

        // Center the flow line
        let padding = (48_i32 - current_line.len() as i32).max(0) / 2;
        diagram.push_str("│");
        diagram.push_str(&" ".repeat(padding as usize));
        diagram.push_str(&current_line);
        diagram.push_str(&" ".repeat((48 - padding as usize - current_line.len()).max(0)));
        diagram.push_str("│\n");

        if self.config.show_types {
            diagram.push_str("├");
            diagram.push_str(&"─".repeat(50));
            diagram.push_str("┤\n");
            diagram.push_str("│ Type Flow:                                       │\n");

            for (i, step) in path.steps.iter().enumerate() {
                let type_line = format!(
                    "  {}: {} → {}",
                    i + 1,
                    step.input_type.description(),
                    step.output_type.description()
                );
                diagram.push_str(&format!("│ {:<49} │\n", type_line));
            }
        }

        diagram.push_str("└");
        diagram.push_str(&"─".repeat(50));
        diagram.push_str("┘\n");

        diagram
    }
}

impl Default for VisualDebugger {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for FlowDiagramGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::ArrowDirection;
    use crate::parser::ArrowFlowStep;
    use crate::types::SusumuType;

    #[test]
    fn test_visual_debugger_creation() {
        let debugger = VisualDebugger::new();
        assert_eq!(debugger.traces.len(), 0);
        assert_eq!(debugger.flow_paths.len(), 0);
    }

    #[test]
    fn test_breakpoint_management() {
        let mut debugger = VisualDebugger::new();

        let bp_id = debugger.set_breakpoint(10, None, None);
        assert_eq!(debugger.breakpoints.len(), 1);

        assert!(debugger.should_break(10, 5));
        assert!(!debugger.should_break(11, 5));

        debugger.remove_breakpoint(10);
        assert_eq!(debugger.breakpoints.len(), 0);
    }

    #[test]
    fn test_flow_diagram_generation() {
        let generator = FlowDiagramGenerator::new();

        let path = ArrowFlowPath {
            start_line: 1,
            start_column: 1,
            steps: vec![
                ArrowFlowStep {
                    expression: "5".to_string(),
                    direction: ArrowDirection::Forward,
                    input_type: SusumuType::Number,
                    output_type: SusumuType::Number,
                    line: 1,
                    column: 1,
                },
                ArrowFlowStep {
                    expression: "add".to_string(),
                    direction: ArrowDirection::Backward,
                    input_type: SusumuType::Number,
                    output_type: SusumuType::Number,
                    line: 1,
                    column: 5,
                },
            ],
            expected_types: vec![SusumuType::Number, SusumuType::Number],
            actual_types: vec![SusumuType::Number, SusumuType::Number],
        };

        let diagram = generator.generate_ascii_diagram(&path);
        assert!(diagram.contains("Arrow Flow Diagram"));
        assert!(diagram.contains("Type Flow"));
    }
}
