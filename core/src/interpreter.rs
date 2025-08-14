//! Interpreter for Susumu with visual debugging and type safety

use crate::ast::*;
use crate::builtins::{value_to_display_string, BuiltinRegistry};
use crate::environment::{Environment, EnvironmentManager};
use crate::error::{SusumuError, SusumuResult};
use serde_json::Value;
use std::sync::Arc;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

// Platform-specific timing imports
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(target_arch = "wasm32")]
use js_sys::Date;

/// Enhanced interpreter with visual debugging and performance optimization
pub struct Interpreter {
    env_manager: EnvironmentManager,
    builtins: BuiltinRegistry,
    // type_checker: TypeChecker,
    /// Visual debugging: execution traces
    execution_traces: Vec<ExecutionTrace>,
    /// Performance monitoring
    performance_stats: PerformanceStats,
}

/// Visual debugging information for execution flow
#[derive(Debug, Clone)]
pub struct ExecutionTrace {
    pub expression: String,
    pub input_value: Value,
    pub output_value: Value,
    pub execution_time_ns: u64,
    pub line: usize,
    pub column: usize,
    pub step_type: ExecutionStepType,
}

#[derive(Debug, Clone)]
pub enum ExecutionStepType {
    ArrowForward {
        from: String,
        to: String,
    },
    ArrowBackward {
        from: String,
        to: String,
    },
    FunctionCall {
        name: String,
        args: Vec<Value>,
    },
    Conditional {
        branch: String,
        condition_result: bool,
    },
    Assignment {
        variable: String,
    },
    Return {
        value: Value,
    },
    Error {
        error: Value,
    },
}

#[derive(Debug, Clone, Default)]
pub struct PerformanceStats {
    pub total_expressions_evaluated: u64,
    pub total_execution_time_ns: u64,
    pub arrow_chain_count: u64,
    pub function_call_count: u64,
    pub convergence_operations: u64,
    pub parallel_operations: u64,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut interpreter = Self {
            env_manager: EnvironmentManager::new(),
            builtins: BuiltinRegistry::new(),
            // type_checker: TypeChecker::new(),
            execution_traces: Vec::new(),
            performance_stats: PerformanceStats::default(),
        };

        interpreter.setup_global_environment();
        interpreter
    }

    /// Execute a program and return the result
    pub fn execute(&mut self, program: &Program) -> SusumuResult<Value> {
        let start_time = self.get_current_time();

        // Register all functions
        for func_def in &program.functions {
            // println!("DEBUG: Registering function: {}", func_def.name);
            self.register_user_function(func_def)?;
        }

        // Execute main expression if present, otherwise try to call main() function
        let result = if let Some(main_expr) = &program.main_expression {
            self.evaluate_with_debugging(main_expr)
        } else {
            // Check if there's a main function and call it automatically
            let global_env = self.env_manager.global();
            if global_env.get_function("main").is_ok() {
                self.call_function_with_args("main", &[], &global_env)
            } else {
                Ok(Value::Null)
            }
        };

        self.performance_stats.total_execution_time_ns = self.elapsed_time_ns(start_time);

        result
    }

    /// Get execution traces for visual debugging
    pub fn get_execution_traces(&self) -> &[ExecutionTrace] {
        &self.execution_traces
    }

    /// Get performance statistics
    pub fn get_performance_stats(&self) -> &PerformanceStats {
        &self.performance_stats
    }

    /// Generate visual execution flow diagram
    pub fn generate_execution_diagram(&self) -> String {
        let mut diagram = String::new();

        diagram.push_str("Execution Flow Diagram:\n");
        diagram.push_str("======================\n\n");

        for (i, trace) in self.execution_traces.iter().enumerate() {
            match &trace.step_type {
                ExecutionStepType::ArrowForward { from, to } => {
                    diagram.push_str(&format!("{}. {} -> {}\n", i + 1, from, to));
                    diagram.push_str(&format!(
                        "   Input:  {}\n",
                        self.value_to_string(&trace.input_value)
                    ));
                    diagram.push_str(&format!(
                        "   Output: {}\n",
                        self.value_to_string(&trace.output_value)
                    ));
                    diagram.push_str(&format!("   Time:   {}ns\n", trace.execution_time_ns));
                }
                ExecutionStepType::ArrowBackward { from, to } => {
                    diagram.push_str(&format!("{}. {} <- {}\n", i + 1, to, from));
                    diagram.push_str(&format!(
                        "   Convergent input: {}\n",
                        self.value_to_string(&trace.input_value)
                    ));
                }
                ExecutionStepType::FunctionCall { name, args } => {
                    diagram.push_str(&format!("{}. {}(", i + 1, name));
                    for (j, arg) in args.iter().enumerate() {
                        if j > 0 {
                            diagram.push_str(", ");
                        }
                        diagram.push_str(&self.value_to_string(arg));
                    }
                    diagram.push_str(")\n");
                    diagram.push_str(&format!(
                        "   Result: {}\n",
                        self.value_to_string(&trace.output_value)
                    ));
                }
                ExecutionStepType::Conditional {
                    branch,
                    condition_result,
                } => {
                    diagram.push_str(&format!(
                        "{}. Conditional: {} ({})\n",
                        i + 1,
                        branch,
                        if *condition_result { "true" } else { "false" }
                    ));
                }
                _ => {
                    diagram.push_str(&format!("{}. {}\n", i + 1, trace.expression));
                }
            }
            diagram.push('\n');
        }

        diagram.push_str(&format!("Performance Summary:\n"));
        diagram.push_str(&format!(
            "  Total expressions: {}\n",
            self.performance_stats.total_expressions_evaluated
        ));
        diagram.push_str(&format!(
            "  Total time: {}ms\n",
            self.performance_stats.total_execution_time_ns / 1_000_000
        ));
        diagram.push_str(&format!(
            "  Arrow chains: {}\n",
            self.performance_stats.arrow_chain_count
        ));
        diagram.push_str(&format!(
            "  Function calls: {}\n",
            self.performance_stats.function_call_count
        ));
        diagram.push_str(&format!(
            "  Convergence ops: {}\n",
            self.performance_stats.convergence_operations
        ));

        diagram
    }

    fn setup_global_environment(&mut self) {
        let global = self.env_manager.global();

        // Register built-in functions as callable values
        for func_name in self.builtins.function_names() {
            global.define(func_name, Value::String("builtin_function".to_string()));
        }
    }

    fn register_user_function(&mut self, func_def: &FunctionDef) -> SusumuResult<()> {
        // Store the function definition in the global environment
        let global = self.env_manager.global();
        global.define_function(func_def.name.clone(), func_def.clone());
        Ok(())
    }

    fn evaluate_with_debugging(&mut self, expr: &Expression) -> SusumuResult<Value> {
        let start_time = self.get_current_time();
        self.performance_stats.total_expressions_evaluated += 1;

        let result = self.evaluate(expr, &self.env_manager.current());

        let execution_time = self.elapsed_time_ns(start_time);

        // Add execution trace for visual debugging
        let trace = ExecutionTrace {
            expression: self.expression_to_debug_string(expr),
            input_value: Value::Null, // Would track actual input
            output_value: result.as_ref().unwrap_or(&Value::Null).clone(),
            execution_time_ns: execution_time,
            line: 1, // Would come from AST metadata
            column: 1,
            step_type: ExecutionStepType::FunctionCall {
                name: "evaluate".to_string(),
                args: vec![],
            },
        };
        self.execution_traces.push(trace);

        result
    }

    fn evaluate(&mut self, expr: &Expression, env: &Arc<Environment>) -> SusumuResult<Value> {
        match expr {
            Expression::Number(n) => Ok(self.create_number_value(*n)),
            Expression::String(s) => Ok(Value::String(s.clone())),
            Expression::Boolean(b) => Ok(Value::Bool(*b)),
            Expression::Null => Ok(Value::Null),

            Expression::Identifier(name) => env
                .get(name)
                .map_err(|_| SusumuError::undefined_variable(name)),

            Expression::Tuple(elements) => {
                let values: Result<Vec<_>, _> =
                    elements.iter().map(|e| self.evaluate(e, env)).collect();
                Ok(Value::Array(values?))
            }

            Expression::Array(elements) => {
                let values: Result<Vec<_>, _> =
                    elements.iter().map(|e| self.evaluate(e, env)).collect();
                Ok(Value::Array(values?))
            }

            Expression::Object(pairs) => {
                let mut object = serde_json::Map::new();
                for (key, value_expr) in pairs {
                    let value = self.evaluate(value_expr, env)?;
                    object.insert(key.clone(), value);
                }
                Ok(Value::Object(object))
            }

            Expression::ArrowChain {
                expressions,
                directions,
            } => self.evaluate_arrow_chain_with_debugging(expressions, directions, env),

            Expression::FunctionCall { name, args } => {
                self.evaluate_function_call_with_debugging(name, args, env)
            }

            Expression::Conditional {
                condition_type,
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => self.evaluate_conditional_with_debugging(
                condition_type,
                condition,
                then_branch,
                else_if_branches,
                else_branch,
                env,
            ),

            Expression::Return(value) => {
                let val = self.evaluate(value, env)?;
                Err(SusumuError::return_value(val))
            }

            Expression::Error(value) => {
                let val = self.evaluate(value, env)?;
                Err(SusumuError::user_error(val))
            }

            Expression::ForEach {
                variable,
                iterable,
                body,
            } => self.evaluate_foreach_with_debugging(variable, iterable, body, env),

            Expression::Block(expressions) => {
                let mut result = Value::Null;
                for expr in expressions {
                    result = self.evaluate(expr, env)?;
                }
                Ok(result)
            }

            Expression::Match { expr, cases } => {
                self.evaluate_match_with_debugging(expr, cases, env)
            }

            Expression::Maybe { value } => match value {
                Some(v) => {
                    let val = self.evaluate(v, env)?;
                    Ok(Value::Object({
                        let mut map = serde_json::Map::new();
                        map.insert("type".to_string(), Value::String("some".to_string()));
                        map.insert("value".to_string(), val);
                        map
                    }))
                }
                None => Ok(Value::Object({
                    let mut map = serde_json::Map::new();
                    map.insert("type".to_string(), Value::String("none".to_string()));
                    map
                })),
            },

            Expression::Result { is_success, value } => {
                let val = self.evaluate(value, env)?;
                Ok(Value::Object({
                    let mut map = serde_json::Map::new();
                    map.insert(
                        "type".to_string(),
                        Value::String(if *is_success { "success" } else { "error" }.to_string()),
                    );
                    map.insert("value".to_string(), val);
                    map
                }))
            }

            Expression::Assignment {
                target,
                value,
                mutable: _,
            } => {
                let val = self.evaluate(value, env)?;
                // Use define to create the variable if it doesn't exist
                // This allows assignments to create new variables
                env.define(target.clone(), val.clone());
                Ok(val)
            }

            Expression::PropertyAccess { object, property } => {
                let obj = self.evaluate(object, env)?;
                Ok(match obj {
                    Value::Object(map) => map.get(property).cloned().unwrap_or(Value::Null),
                    _ => Value::Null,
                })
            }

            Expression::BinaryOp {
                left,
                operator,
                right,
            } => {
                let left_val = self.evaluate(left, env)?;
                let right_val = self.evaluate(right, env)?;
                self.evaluate_binary_op(&left_val, operator, &right_val)
            }

            Expression::Annotated {
                annotation,
                expression,
            } => self.evaluate_annotated_expression(annotation, expression, env),
        }
    }

    fn evaluate_annotated_expression(
        &mut self,
        annotation: &Annotation,
        expression: &Expression,
        env: &Arc<Environment>,
    ) -> SusumuResult<Value> {
        // Handle the annotation and then evaluate the expression
        match annotation {
            Annotation::Trace(trace_name) => {
                println!("🔍 TRACE [{}]: Starting execution", trace_name);
                let start_time = self.get_current_time();
                let result = self.evaluate(expression, env);
                let end_time = self.get_current_time();

                match &result {
                    Ok(value) => println!(
                        "✅ TRACE [{}]: Completed in {}ns -> {}",
                        trace_name,
                        self.calculate_duration(start_time, end_time),
                        value_to_display_string(value)
                    ),
                    Err(error) => println!("❌ TRACE [{}]: Error -> {}", trace_name, error),
                }
                result
            }
            Annotation::Monitor(metrics) => {
                let start_time = self.get_current_time();
                let result = self.evaluate(expression, env);
                let end_time = self.get_current_time();
                let duration = self.calculate_duration(start_time, end_time);

                for metric in metrics {
                    match metric.as_str() {
                        "latency" => println!("⏱️  MONITOR latency: {}ns", duration),
                        "errors" => {
                            if result.is_err() {
                                println!("🚨 MONITOR errors: execution failed");
                            }
                        }
                        "memory" => println!("💾 MONITOR memory: tracking not yet implemented"),
                        _ => println!("📊 MONITOR {}: tracking not yet implemented", metric),
                    }
                }
                result
            }
            Annotation::Config(config) => {
                println!("⚙️  CONFIG: Applying configuration: {}", config);
                // For now, just evaluate normally - config could control execution behavior
                self.evaluate(expression, env)
            }
            Annotation::Parallel => {
                println!("🏃‍♂️ PARALLEL: Marking expression for parallel execution");
                // For now, evaluate normally - parallel would be handled in arrow chains
                self.evaluate(expression, env)
            }
            Annotation::Debug(label) => {
                match label {
                    Some(checkpoint) => {
                        println!("🐞 DEBUG [{}]: Starting debug checkpoint", checkpoint)
                    }
                    None => println!("🐞 DEBUG: Starting debug execution"),
                }

                // Add detailed debugging
                println!("   Expression: {:?}", expression);
                let result = self.evaluate(expression, env);

                match &result {
                    Ok(value) => match label {
                        Some(checkpoint) => println!(
                            "🐞 DEBUG [{}]: Result -> {}",
                            checkpoint,
                            value_to_display_string(value)
                        ),
                        None => println!("🐞 DEBUG: Result -> {}", value_to_display_string(value)),
                    },
                    Err(error) => match label {
                        Some(checkpoint) => {
                            println!("🐞 DEBUG [{}]: Error -> {}", checkpoint, error)
                        }
                        None => println!("🐞 DEBUG: Error -> {}", error),
                    },
                }
                result
            }
        }
    }

    fn evaluate_arrow_chain_with_debugging(
        &mut self,
        expressions: &[Expression],
        directions: &[ArrowDirection],
        env: &Arc<Environment>,
    ) -> SusumuResult<Value> {
        self.performance_stats.arrow_chain_count += 1;

        if expressions.is_empty() {
            return Ok(Value::Null);
        }

        if directions.is_empty() {
            return self.evaluate(&expressions[0], env);
        }

        // Implement convergence semantics: x -> func <- y <- z means func(x, y, z)
        let mut result = self.evaluate(&expressions[0], env)?;
        let mut i = 0;

        while i < directions.len() {
            let direction = &directions[i];
            let current_expr = &expressions[i + 1];

            match direction {
                ArrowDirection::Forward => {
                    if let Expression::Identifier(func_name) = current_expr {
                        // Check for convergent arguments
                        let mut args = vec![result.clone()];
                        let mut j = i + 1;

                        // Collect convergent expressions for parallel evaluation
                        let mut convergent_expressions = Vec::new();
                        while j < directions.len() && directions[j] == ArrowDirection::Backward {
                            j += 1;
                            if j < expressions.len() {
                                convergent_expressions.push(&expressions[j]);
                            }
                        }

                        // Evaluate convergent arguments - use parallel processing if available
                        #[cfg(feature = "parallel")]
                        {
                            if convergent_expressions.len() > 1 {
                                // Parallel evaluation using rayon - significant performance boost for convergent operations
                                let env_clone = env.clone();
                                let env_manager_clone = self.env_manager.clone();
                                let builtins_clone = self.builtins.clone();

                                let convergent_results: Result<Vec<Value>, SusumuError> =
                                    convergent_expressions
                                        .par_iter()
                                        .map(|expr| {
                                            // Each thread gets its own interpreter instance with shared state
                                            let mut temp_interpreter = Interpreter::new();
                                            temp_interpreter.env_manager =
                                                env_manager_clone.clone();
                                            temp_interpreter.builtins = builtins_clone.clone();
                                            temp_interpreter.evaluate(expr, &env_clone)
                                        })
                                        .collect();

                                match convergent_results {
                                    Ok(results) => args.extend(results),
                                    Err(e) => return Err(e),
                                }

                                self.performance_stats.parallel_operations += 1;
                            } else {
                                // Single or no convergent arguments
                                for expr in convergent_expressions {
                                    let converging_value = self.evaluate(expr, env)?;
                                    args.push(converging_value);
                                }
                            }
                        }

                        #[cfg(not(feature = "parallel"))]
                        {
                            // Sequential evaluation fallback
                            for expr in convergent_expressions {
                                let converging_value = self.evaluate(expr, env)?;
                                args.push(converging_value);
                            }
                        }

                        self.performance_stats.convergence_operations += 1;

                        // Add debugging trace for convergence
                        let _trace = ExecutionTrace {
                            expression: func_name.clone(),
                            input_value: Value::Array(args.clone()),
                            output_value: Value::Null, // Will be updated
                            execution_time_ns: 0,
                            line: 1,
                            column: 1,
                            step_type: ExecutionStepType::ArrowForward {
                                from: self.value_to_string(&result),
                                to: func_name.clone(),
                            },
                        };

                        result = self.call_function_with_args(func_name, &args, env)?;

                        // Update trace with result
                        if let Some(last_trace) = self.execution_traces.last_mut() {
                            last_trace.output_value = result.clone();
                        }

                        i = j; // Skip past convergence
                    } else if let Expression::Match { expr, cases } = current_expr {
                        // Special handling for match expressions in arrow chains
                        // Pass the current result as the value to match against
                        match expr {
                            Some(_) => {
                                // Match expression has its own expression, evaluate normally
                                result = self.evaluate(current_expr, env)?;
                            }
                            None => {
                                // Match expression expects input from arrow chain
                                result = self.evaluate_match_with_value(&result, cases, env)?;
                            }
                        }
                        i += 1;
                    } else if let Expression::Conditional {
                        condition_type,
                        condition,
                        then_branch,
                        else_if_branches,
                        else_branch,
                    } = current_expr
                    {
                        // Special handling for conditional expressions in arrow chains
                        // Use the current arrow chain result as the condition
                        if let Expression::Null = **condition {
                            // This is a conditional from an arrow chain (placeholder condition)
                            // Use the current result as the condition
                            result = self.evaluate_conditional_with_arrow_result(
                                condition_type,
                                &result,
                                then_branch,
                                else_if_branches,
                                else_branch,
                                env,
                            )?;
                        } else {
                            // Normal conditional evaluation
                            result = self.evaluate(current_expr, env)?;
                        }
                        i += 1;
                    } else {
                        // Direct function call or other expression
                        result = self.evaluate(current_expr, env)?;
                        i += 1;
                    }
                }
                ArrowDirection::Backward => {
                    // Backward arrows are handled in the forward arrow convergence logic
                    return Err(SusumuError::arrow_chain_error(
                        "Unexpected backward arrow - convergence should be handled by forward arrow processing"
                    ));
                }
            }
        }

        Ok(result)
    }

    fn evaluate_conditional_with_arrow_result(
        &mut self,
        condition_type: &ConditionType,
        arrow_result: &Value,
        then_branch: &Expression,
        else_if_branches: &Vec<ElseIfBranch>,
        else_branch: &Option<Box<Expression>>,
        env: &Arc<Environment>,
    ) -> SusumuResult<Value> {
        let mut branch_name = "none";
        let result;

        // Check main condition using arrow result
        let branch_taken = match condition_type {
            ConditionType::Success => !matches!(arrow_result, Value::Null),
            ConditionType::Custom(condition_name) => {
                self.evaluate_custom_condition(condition_name, arrow_result)?
            }
            ConditionType::If => self.is_truthy(arrow_result),
        };

        if branch_taken {
            result = self.evaluate(then_branch, env)?;
            branch_name = "then";
        } else {
            // Check else-if branches
            let mut else_if_taken = false;
            let mut else_if_result = Value::Null;

            for else_if_branch in else_if_branches {
                let else_if_condition_result = match &else_if_branch.condition_type {
                    ConditionType::Success => !matches!(arrow_result, Value::Null),
                    ConditionType::Custom(condition_name) => {
                        self.evaluate_custom_condition(condition_name, arrow_result)?
                    }
                    ConditionType::If => self.is_truthy(arrow_result),
                };

                if else_if_condition_result {
                    else_if_result = self.evaluate(&else_if_branch.then_branch, env)?;
                    else_if_taken = true;
                    branch_name = "else-if";
                    break;
                }
            }

            if else_if_taken {
                result = else_if_result;
            } else if let Some(else_expr) = else_branch {
                result = self.evaluate(else_expr, env)?;
                branch_name = "else";
            } else {
                result = Value::Null;
            }
        }

        // Add execution trace
        let trace = ExecutionTrace {
            expression: "arrow-conditional".to_string(),
            input_value: arrow_result.clone(),
            output_value: result.clone(),
            execution_time_ns: 0,
            line: 1,
            column: 1,
            step_type: ExecutionStepType::Conditional {
                branch: branch_name.to_string(),
                condition_result: branch_taken || branch_name != "none",
            },
        };
        self.execution_traces.push(trace);

        Ok(result)
    }

    fn evaluate_function_call_with_debugging(
        &mut self,
        name: &str,
        args: &[Expression],
        env: &Arc<Environment>,
    ) -> SusumuResult<Value> {
        self.performance_stats.function_call_count += 1;

        let arg_values: Result<Vec<_>, _> =
            args.iter().map(|arg| self.evaluate(arg, env)).collect();
        let arg_values = arg_values?;

        let trace = ExecutionTrace {
            expression: format!("{}()", name),
            input_value: Value::Array(arg_values.clone()),
            output_value: Value::Null,
            execution_time_ns: 0,
            line: 1,
            column: 1,
            step_type: ExecutionStepType::FunctionCall {
                name: name.to_string(),
                args: arg_values.clone(),
            },
        };
        self.execution_traces.push(trace);

        self.call_function_with_args(name, &arg_values, env)
    }

    fn evaluate_conditional_with_debugging(
        &mut self,
        condition_type: &ConditionType,
        condition: &Expression,
        then_branch: &Expression,
        else_if_branches: &Vec<ElseIfBranch>,
        else_branch: &Option<Box<Expression>>,
        env: &Arc<Environment>,
    ) -> SusumuResult<Value> {
        let condition_value = self.evaluate(condition, env)?;
        let mut branch_name = "none";
        let result;

        // Check main condition
        let branch_taken = match condition_type {
            ConditionType::Success => !matches!(condition_value, Value::Null),
            ConditionType::Custom(condition_name) => {
                self.evaluate_custom_condition(condition_name, &condition_value)?
            }
            ConditionType::If => self.is_truthy(&condition_value),
        };

        if branch_taken {
            result = self.evaluate(then_branch, env)?;
            branch_name = "then";
        } else {
            // Check else-if branches
            let mut else_if_taken = false;
            let mut else_if_result = Value::Null;

            for else_if_branch in else_if_branches {
                let else_if_condition_result = match &else_if_branch.condition_type {
                    ConditionType::Success => !matches!(condition_value, Value::Null),
                    ConditionType::Custom(condition_name) => {
                        self.evaluate_custom_condition(condition_name, &condition_value)?
                    }
                    ConditionType::If => self.is_truthy(&condition_value),
                };

                if else_if_condition_result {
                    else_if_result = self.evaluate(&else_if_branch.then_branch, env)?;
                    else_if_taken = true;
                    branch_name = "else-if";
                    break;
                }
            }

            if else_if_taken {
                result = else_if_result;
            } else if let Some(else_expr) = else_branch {
                result = self.evaluate(else_expr, env)?;
                branch_name = "else";
            } else {
                result = Value::Null;
            }
        }

        let trace = ExecutionTrace {
            expression: "conditional".to_string(),
            input_value: condition_value,
            output_value: result.clone(),
            execution_time_ns: 0,
            line: 1,
            column: 1,
            step_type: ExecutionStepType::Conditional {
                branch: branch_name.to_string(),
                condition_result: branch_taken || branch_name != "none",
            },
        };
        self.execution_traces.push(trace);

        Ok(result)
    }

    fn evaluate_foreach_with_debugging(
        &mut self,
        variable: &str,
        iterable: &Expression,
        body: &Expression,
        env: &Arc<Environment>,
    ) -> SusumuResult<Value> {
        let iterable_value = self.evaluate(iterable, env)?;

        match iterable_value {
            Value::Array(items) => {
                let mut results = Vec::new();

                for item in items {
                    // Create new scope for loop iteration
                    let new_scope = self.env_manager.push_scope();
                    new_scope.define(variable.to_string(), item.clone());
                    let loop_result = self.evaluate(body, &new_scope)?;
                    self.env_manager.pop_scope()?;

                    results.push(loop_result);
                }

                Ok(Value::Array(results))
            }
            _ => Err(SusumuError::type_error(
                "array",
                &format!("{:?}", iterable_value),
            )),
        }
    }

    fn call_function_with_args(
        &mut self,
        name: &str,
        args: &[Value],
        env: &Arc<Environment>,
    ) -> SusumuResult<Value> {
        // Try built-in functions first
        if self.builtins.contains(name) {
            return self.builtins.call(name, args);
        }

        // Try user-defined functions in global environment
        let global_env = self.env_manager.global();
        if let Ok(func_def) = global_env.get_function(name) {
            return self.call_user_function(&func_def, args, env);
        }

        Err(SusumuError::undefined_function(name))
    }

    fn call_user_function(
        &mut self,
        func_def: &crate::ast::FunctionDef,
        args: &[Value],
        _parent_env: &Arc<Environment>,
    ) -> SusumuResult<Value> {
        if args.len() != func_def.params.len() {
            return Err(SusumuError::function_call_error(&format!(
                "Function {} expects {} arguments, got {}",
                func_def.name,
                func_def.params.len(),
                args.len()
            )));
        }

        // Create new scope for function execution
        let func_scope = self.env_manager.push_scope();

        // Bind parameters to arguments
        for (param, arg) in func_def.params.iter().zip(args.iter()) {
            func_scope.define(param.clone(), arg.clone());
        }

        // Execute function body
        let result = match self.evaluate(&func_def.body, &func_scope) {
            Ok(result) => Ok(result),
            Err(SusumuError::ReturnValue { value }) => Ok(value),
            Err(other) => Err(other),
        };

        self.env_manager.pop_scope()?;
        result
    }

    fn evaluate_custom_condition(&self, condition_name: &str, value: &Value) -> SusumuResult<bool> {
        match condition_name {
            "success" => Ok(!matches!(value, Value::Null)),
            "null" => Ok(matches!(value, Value::Null)),
            "empty" => Ok(match value {
                Value::String(s) => s.is_empty(),
                Value::Array(a) => a.is_empty(),
                _ => false,
            }),
            _ => {
                // For custom conditions, check if it's a truthy value
                Ok(self.is_truthy(value))
            }
        }
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::Object(o) => !o.is_empty(),
        }
    }

    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            Value::Array(a) => format!("[{} items]", a.len()),
            Value::Object(o) => format!("{{object with {} fields}}", o.len()),
        }
    }

    fn expression_to_debug_string(&self, expr: &Expression) -> String {
        match expr {
            Expression::Identifier(name) => name.clone(),
            Expression::Number(n) => n.to_string(),
            Expression::String(s) => format!("\"{}\"", s),
            Expression::Boolean(b) => b.to_string(),
            Expression::Null => "null".to_string(),
            Expression::ArrowChain {
                expressions,
                directions,
            } => {
                let mut result = String::new();
                result.push_str(&self.expression_to_debug_string(&expressions[0]));

                for (i, direction) in directions.iter().enumerate() {
                    let arrow = match direction {
                        ArrowDirection::Forward => " -> ",
                        ArrowDirection::Backward => " <- ",
                    };
                    result.push_str(arrow);
                    result.push_str(&self.expression_to_debug_string(&expressions[i + 1]));
                }
                result
            }
            Expression::FunctionCall { name, args } => {
                format!("{}({} args)", name, args.len())
            }
            _ => "expression".to_string(),
        }
    }

    // Platform-specific timing methods
    #[cfg(not(target_arch = "wasm32"))]
    fn get_current_time(&self) -> Instant {
        Instant::now()
    }

    #[cfg(target_arch = "wasm32")]
    fn get_current_time(&self) -> f64 {
        Date::now()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn elapsed_time_ns(&self, start_time: Instant) -> u64 {
        start_time.elapsed().as_nanos() as u64
    }

    #[cfg(target_arch = "wasm32")]
    fn elapsed_time_ns(&self, start_time: f64) -> u64 {
        let elapsed_ms = Date::now() - start_time;
        (elapsed_ms * 1_000_000.0) as u64 // Convert ms to ns
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn calculate_duration(&self, start_time: Instant, end_time: Instant) -> u64 {
        (end_time - start_time).as_nanos() as u64
    }

    #[cfg(target_arch = "wasm32")]
    fn calculate_duration(&self, start_time: f64, end_time: f64) -> u64 {
        let elapsed_ms = end_time - start_time;
        (elapsed_ms * 1_000_000.0) as u64 // Convert ms to ns
    }

    /// Helper function to create JSON numbers that preserve integer type when appropriate
    fn create_number_value(&self, value: f64) -> Value {
        if value.fract() == 0.0
            && value.is_finite()
            && value >= i64::MIN as f64
            && value <= i64::MAX as f64
        {
            // Return as integer if it's a whole number within i64 range
            serde_json::json!(value as i64)
        } else {
            // Return as float
            serde_json::json!(value)
        }
    }

    fn evaluate_match_with_debugging(
        &mut self,
        expr: &Option<Box<Expression>>,
        cases: &[MatchCase],
        env: &Arc<Environment>,
    ) -> SusumuResult<Value> {
        let value = match expr {
            Some(e) => {
                let result = self.evaluate(e, env)?;
                result
            }
            None => {
                Value::Null // For arrow chain integration
            }
        };

        for case in cases {
            if let Some(bindings) = self.match_pattern(&case.pattern, &value) {
                // Check guard condition if present
                if let Some(guard) = &case.guard {
                    // Create new scope with pattern bindings
                    let new_env = Arc::new(Environment::with_parent(env.clone()));
                    for (name, val) in &bindings {
                        new_env.set(name, val.clone())?;
                    }

                    let guard_result = self.evaluate(guard, &new_env)?;
                    if !self.is_truthy(&guard_result) {
                        continue;
                    }
                }

                // Execute the match case body with bindings
                let new_env = Arc::new(Environment::with_parent(env.clone()));
                for (name, val) in bindings {
                    new_env.define(name, val);
                }
                return self.evaluate(&case.body, &new_env);
            }
        }

        Err(SusumuError::runtime_error("No pattern matched"))
    }

    fn evaluate_match_with_value(
        &mut self,
        value: &Value,
        cases: &[MatchCase],
        env: &Arc<Environment>,
    ) -> SusumuResult<Value> {
        for case in cases {
            if let Some(bindings) = self.match_pattern(&case.pattern, value) {
                // Check guard condition if present
                if let Some(guard) = &case.guard {
                    // Create new scope with pattern bindings
                    let new_env = Arc::new(Environment::with_parent(env.clone()));
                    for (name, val) in &bindings {
                        new_env.set(name, val.clone())?;
                    }

                    let guard_result = self.evaluate(guard, &new_env)?;
                    if !self.is_truthy(&guard_result) {
                        continue;
                    }
                }

                // Execute the match case body with bindings
                let new_env = Arc::new(Environment::with_parent(env.clone()));
                for (name, val) in bindings {
                    new_env.define(name, val);
                }
                return self.evaluate(&case.body, &new_env);
            }
        }

        Err(SusumuError::runtime_error("No pattern matched"))
    }

    fn match_pattern(
        &self,
        pattern: &Pattern,
        value: &Value,
    ) -> Option<std::collections::HashMap<String, Value>> {
        use std::collections::HashMap;
        let mut bindings = HashMap::new();

        match pattern {
            Pattern::Wildcard => Some(bindings),
            Pattern::Identifier(name) => {
                bindings.insert(name.clone(), value.clone());
                Some(bindings)
            }
            Pattern::Literal(lit) => {
                if self.literal_matches(lit, value) {
                    Some(bindings)
                } else {
                    None
                }
            }
            Pattern::ArrowPattern { constructor, arg } => {
                if let Value::Object(map) = value {
                    if let Some(Value::String(type_str)) = map.get("type") {
                        if type_str == constructor {
                            if let Some(inner_value) = map.get("value") {
                                if let Some(inner_bindings) = self.match_pattern(arg, inner_value) {
                                    bindings.extend(inner_bindings);
                                    Some(bindings)
                                } else {
                                    None
                                }
                            } else {
                                // Constructor without value (like "none")
                                Some(bindings)
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Pattern::Tuple(patterns) => {
                if let Value::Array(values) = value {
                    if patterns.len() == values.len() {
                        for (pattern, value) in patterns.iter().zip(values.iter()) {
                            if let Some(inner_bindings) = self.match_pattern(pattern, value) {
                                bindings.extend(inner_bindings);
                            } else {
                                return None;
                            }
                        }
                        Some(bindings)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Pattern::Object(field_patterns) => {
                if let Value::Object(obj) = value {
                    for (key, pattern) in field_patterns {
                        if let Some(field_value) = obj.get(key) {
                            if let Some(inner_bindings) = self.match_pattern(pattern, field_value) {
                                bindings.extend(inner_bindings);
                            } else {
                                return None;
                            }
                        } else {
                            return None;
                        }
                    }
                    Some(bindings)
                } else {
                    None
                }
            }
        }
    }

    fn literal_matches(&self, literal: &LiteralValue, value: &Value) -> bool {
        let result = match (literal, value) {
            (LiteralValue::Number(n1), Value::Number(n2)) => {
                let n2_f64 = n2.as_f64().unwrap_or(0.0);
                n1 == &n2_f64
            }
            (LiteralValue::String(s1), Value::String(s2)) => s1 == s2,
            (LiteralValue::Boolean(b1), Value::Bool(b2)) => b1 == b2,
            (LiteralValue::Null, Value::Null) => true,
            _ => false,
        };
        result
    }

    fn evaluate_binary_op(
        &self,
        left: &Value,
        operator: &BinaryOperator,
        right: &Value,
    ) -> SusumuResult<Value> {
        use BinaryOperator::*;

        match (left, right) {
            (Value::Number(a), Value::Number(b)) => {
                let a_val = a.as_f64().unwrap_or(0.0);
                let b_val = b.as_f64().unwrap_or(0.0);

                match operator {
                    Add => Ok(self.create_number_value(a_val + b_val)),
                    Subtract => Ok(self.create_number_value(a_val - b_val)),
                    Multiply => Ok(self.create_number_value(a_val * b_val)),
                    Divide => Ok(self.create_number_value(a_val / b_val)),
                    Equal => Ok(Value::Bool(a_val == b_val)),
                    NotEqual => Ok(Value::Bool(a_val != b_val)),
                    Less => Ok(Value::Bool(a_val < b_val)),
                    Greater => Ok(Value::Bool(a_val > b_val)),
                    LessEq => Ok(Value::Bool(a_val <= b_val)),
                    GreaterEq => Ok(Value::Bool(a_val >= b_val)),
                }
            }
            (Value::String(a), Value::String(b)) => match operator {
                Add => Ok(Value::String(format!("{}{}", a, b))),
                Equal => Ok(Value::Bool(a == b)),
                NotEqual => Ok(Value::Bool(a != b)),
                _ => Err(SusumuError::runtime_error(&format!(
                    "Unsupported operation {:?} on strings",
                    operator
                ))),
            },
            (Value::String(a), Value::Number(b)) => match operator {
                Add => Ok(Value::String(format!("{}{}", a, self.number_to_string(b)))),
                _ => Err(SusumuError::runtime_error(&format!(
                    "Unsupported operation {:?} on string and number",
                    operator
                ))),
            },
            (Value::Number(a), Value::String(b)) => match operator {
                Add => Ok(Value::String(format!("{}{}", self.number_to_string(a), b))),
                _ => Err(SusumuError::runtime_error(&format!(
                    "Unsupported operation {:?} on number and string",
                    operator
                ))),
            },
            _ => match operator {
                Equal => Ok(Value::Bool(format!("{:?}", left) == format!("{:?}", right))),
                NotEqual => Ok(Value::Bool(format!("{:?}", left) != format!("{:?}", right))),
                _ => Err(SusumuError::runtime_error(&format!(
                    "Unsupported operation {:?} on these types",
                    operator
                ))),
            },
        }
    }

    fn number_to_string(&self, number: &serde_json::Number) -> String {
        if let Some(i) = number.as_i64() {
            i.to_string()
        } else if let Some(f) = number.as_f64() {
            if f.fract() == 0.0 {
                (f as i64).to_string()
            } else {
                f.to_string()
            }
        } else {
            number.to_string()
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn test_basic_arithmetic() {
        let source = "5 -> add <- 3";
        let tokens = Lexer::new(source).tokenize().unwrap();
        let ast = Parser::new(tokens).parse().unwrap();
        let mut interpreter = Interpreter::new();

        let result = interpreter.execute(&ast).unwrap();
        assert_eq!(result, Value::Number(serde_json::Number::from(8)));
    }

    #[test]
    fn test_convergence_semantics() {
        let source = "5 -> add <- 3 <- 2"; // Should be 5 + 3 + 2 = 10
        let tokens = Lexer::new(source).tokenize().unwrap();
        let ast = Parser::new(tokens).parse().unwrap();
        let mut interpreter = Interpreter::new();

        let result = interpreter.execute(&ast).unwrap();
        assert_eq!(result, Value::Number(serde_json::Number::from(10)));
    }

    #[test]
    fn test_visual_debugging_traces() {
        let source = "5 -> add <- 3 -> print";
        let tokens = Lexer::new(source).tokenize().unwrap();
        let ast = Parser::new(tokens).parse().unwrap();
        let mut interpreter = Interpreter::new();

        let _result = interpreter.execute(&ast).unwrap();

        let traces = interpreter.get_execution_traces();
        assert!(!traces.is_empty());

        let diagram = interpreter.generate_execution_diagram();
        assert!(diagram.contains("Execution Flow Diagram"));
        assert!(diagram.contains("Performance Summary"));
    }

    #[test]
    fn test_conditional_execution() {
        let source = r#"
        main() {
            value = 8
            value -> i positive {
                "positive" -> print
            } e {
                "not positive" -> print
            }
        }
        "#;
        let tokens = Lexer::new(source).tokenize().unwrap();
        let ast = Parser::new(tokens).parse().unwrap();
        let mut interpreter = Interpreter::new();

        let result = interpreter.execute(&ast);
        assert!(result.is_ok());
    }
}
