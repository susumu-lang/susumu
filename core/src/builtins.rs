//! Built-in functions for Susumu language

use crate::error::{SusumuError, SusumuResult};
use serde_json::{json, Value};
use std::collections::HashMap;

pub type BuiltinFunction = fn(&[Value]) -> SusumuResult<Value>;

/// Registry of all built-in functions
#[derive(Clone)]
pub struct BuiltinRegistry {
    functions: HashMap<String, BuiltinFunction>,
}

impl BuiltinRegistry {
    /// Create a new builtin registry with all standard functions
    pub fn new() -> Self {
        let mut registry = Self {
            functions: HashMap::new(),
        };

        // Module system functions only (old functions removed to avoid conflicts)
        registry.register_module_functions();

        // Auto-available core functions (hybrid approach) - registered last to override placeholders
        registry.register_core_functions();

        registry
    }

    /// Register a builtin function
    pub fn register(&mut self, name: &str, func: BuiltinFunction) {
        self.functions.insert(name.to_string(), func);
    }

    /// Call a builtin function
    pub fn call(&self, name: &str, args: &[Value]) -> SusumuResult<Value> {
        if let Some(func) = self.functions.get(name) {
            func(args)
        } else {
            Err(SusumuError::undefined_function(name))
        }
    }

    /// Check if a function is a builtin
    pub fn contains(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Get all builtin function names
    pub fn function_names(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }

    fn register_math_functions(&mut self) {
        self.register("add", builtin_add);
        self.register("subtract", builtin_subtract);
        self.register("multiply", builtin_multiply);
        self.register("divide", builtin_divide);
        self.register("modulo", builtin_modulo);
        self.register("power", builtin_power);
        self.register("sqrt", builtin_sqrt);
        self.register("abs", builtin_abs);
        self.register("min", builtin_min);
        self.register("max", builtin_max);
        self.register("sum", builtin_sum);
        self.register("average", builtin_average);
    }

    fn register_string_functions(&mut self) {
        self.register("concat", builtin_concat);
        self.register("length", builtin_length);
        self.register("substring", builtin_substring);
        self.register("to_upper", builtin_to_upper);
        self.register("to_lower", builtin_to_lower);
        self.register("trim", builtin_trim);
        self.register("split", builtin_split);
        self.register("contains", builtin_contains);
    }

    fn register_array_functions(&mut self) {
        self.register("first", builtin_first);
        self.register("last", builtin_last);
        self.register("rest", builtin_rest);
        self.register("push", builtin_push);
        self.register("pop", builtin_pop);
        self.register("filter", builtin_filter);
        self.register("map", builtin_map);
        self.register("reduce", builtin_reduce);
        self.register("sort", builtin_sort);
        self.register("reverse", builtin_reverse);
    }

    fn register_io_functions(&mut self) {
        self.register("print", builtin_print);
        self.register("println", builtin_println);
        self.register("debug", builtin_debug);
    }

    fn register_utility_functions(&mut self) {
        self.register("type_of", builtin_type_of);
        self.register("is_null", builtin_is_null);
        self.register("is_number", builtin_is_number);
        self.register("is_string", builtin_is_string);
        self.register("is_boolean", builtin_is_boolean);
        self.register("is_array", builtin_is_array);
        self.register("is_object", builtin_is_object);
        self.register("to_string", builtin_to_string);
        self.register("to_number", builtin_to_number);
    }

    fn register_maybe_result_functions(&mut self) {
        self.register("some", builtin_some);
        self.register("none", builtin_none);
        self.register("success", builtin_success);
        self.register("error", builtin_error);
        self.register("is_some", builtin_is_some);
        self.register("is_none", builtin_is_none);
        self.register("is_success", builtin_is_success);
        self.register("is_error", builtin_is_error);
        self.register("unwrap", builtin_unwrap);
        self.register("unwrap_or", builtin_unwrap_or);
        self.register("combine", builtin_combine);
    }

    /// Register core auto-available functions (hybrid approach)
    /// These are immediately available without imports for maximum productivity
    fn register_core_functions(&mut self) {
        // === CORE MATH (Auto-available) ===
        self.register("add", builtin_core_add);
        self.register("subtract", builtin_core_subtract);
        self.register("multiply", builtin_core_multiply);
        self.register("divide", builtin_core_divide);
        self.register("addNumbers", builtin_core_add); // Alias for compatibility
        self.register("multiplyNumbers", builtin_core_multiply);

        // === CORE I/O (Auto-available) ===
        self.register("print", builtin_core_print);
        self.register("println", builtin_core_println);

        // === CORE CONVERSIONS (Auto-available) ===
        self.register("toString", builtin_core_to_string);
        self.register("toNumber", builtin_core_to_number);

        // === CORE VALIDATION (Auto-available) ===
        self.register("isNull", builtin_core_is_null);
        self.register("isEmpty", builtin_core_is_empty);
        self.register("isNumber", builtin_core_is_number);
        self.register("isString", builtin_core_is_string);
        self.register("isArray", builtin_core_is_array);

        // === CORE UTILITIES (Auto-available) ===
        self.register("length", builtin_core_length);
        self.register("type", builtin_core_type);
        self.register("equals", builtin_core_equals);

        // === CORE I/O FUNCTIONS (Auto-available for productivity) ===
        self.register("readFile", builtin_core_read_file);
        self.register("writeFile", builtin_core_write_file);
        self.register("appendFile", builtin_core_append_file);
        self.register("fileExists", builtin_core_file_exists);
        self.register("fileInfo", builtin_core_file_info);
        self.register("listDir", builtin_core_list_dir);

        // === CORE JSON/DATA PROCESSING (Auto-available) ===
        self.register("parseJSON", builtin_core_parse_json);
        self.register("toJSON", builtin_core_to_json);

        // === CORE ARRAY PROCESSING (Auto-available) ===
        self.register("filter", builtin_core_filter);
        self.register("map", builtin_core_map);
        self.register("reduce", builtin_core_reduce);

        // === CORE DATE/TIME FUNCTIONS (Auto-available) ===
        self.register("now", builtin_core_now);
        self.register("nowMillis", builtin_core_now_millis);
        self.register("formatDate", builtin_core_format_date);
        self.register("parseDate", builtin_core_parse_date);
        self.register("addTime", builtin_core_add_time);

        // === STDLIB IMPLEMENTATION FUNCTIONS ===
        // Math module implementations
        self.register("calculate_factorial", builtin_calculate_factorial);
        self.register("calculate_gcd", builtin_calculate_gcd);
        self.register("check_prime_factors", builtin_check_prime_factors);
        self.register("calculate_fibonacci", builtin_calculate_fibonacci);

        // String module implementations
        self.register("split_string", builtin_split_string);
        self.register("join_array", builtin_join_array);
        self.register("to_title_case", builtin_to_title_case);

        // Core math functions
        self.register("abs", builtin_core_abs);

        // Core array functions
        self.register("first", builtin_core_first);
        self.register("last", builtin_core_last);
        self.register("reverse", builtin_core_reverse);
        self.register("sum", builtin_core_sum);
        self.register("timeDiff", builtin_core_time_diff);

        // === CORE HTTP CLIENT (Auto-available) ===
        self.register("httpGet", builtin_core_http_get);
        self.register("httpPost", builtin_core_http_post);
        self.register("httpRequest", builtin_core_http_request);
        self.register("httpGetParallel", builtin_core_http_get_parallel);
        self.register("httpPostParallel", builtin_core_http_post_parallel);

        // === PARALLEL OPERATIONS (Auto-available for performance) ===
        self.register("readFilesParallel", builtin_core_read_files_parallel);
        self.register("mapParallel", builtin_core_map_parallel);

        // === STDLIB SUPPORT FUNCTIONS (Auto-available for modules) ===
        self.register("toRadians", builtin_to_radians);
        self.register("applySin", builtin_apply_sin);
        self.register("applyCos", builtin_apply_cos);
        self.register("applyTan", builtin_apply_tan);
        self.register("lessThanOrEqual", builtin_less_than_or_equal);
        self.register("modulo", builtin_modulo);
        self.register("sqrt", builtin_sqrt);
        self.register("power", builtin_power);
        self.register("checkPrimeFactors", builtin_check_prime_factors);
        self.register("performSplit", builtin_perform_split);
        self.register("splitIntoChunks", builtin_split_into_chunks);
    }
}

impl Default for BuiltinRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create JSON numbers that preserve integer type when appropriate
fn create_number_value(value: f64) -> Value {
    if value.fract() == 0.0
        && value.is_finite()
        && value >= i64::MIN as f64
        && value <= i64::MAX as f64
    {
        // Return as integer if it's a whole number within i64 range
        json!(value as i64)
    } else {
        // Return as float
        json!(value)
    }
}

// Math functions
fn builtin_add(args: &[Value]) -> SusumuResult<Value> {
    if args.is_empty() {
        return Err(SusumuError::function_call_error(
            "add requires at least one argument",
        ));
    }

    let mut result = 0.0;
    for arg in args {
        match arg {
            Value::Number(n) => result += n.as_f64().unwrap(),
            _ => return Err(SusumuError::type_error("number", &format!("{:?}", arg))),
        }
    }
    Ok(create_number_value(result))
}

fn builtin_subtract(args: &[Value]) -> SusumuResult<Value> {
    match args.len() {
        1 => match &args[0] {
            Value::Number(n) => Ok(create_number_value(-n.as_f64().unwrap())),
            _ => Err(SusumuError::type_error("number", &format!("{:?}", args[0]))),
        },
        2 => match (&args[0], &args[1]) {
            (Value::Number(a), Value::Number(b)) => Ok(create_number_value(
                a.as_f64().unwrap() - b.as_f64().unwrap(),
            )),
            _ => Err(SusumuError::type_error("number", "non-number")),
        },
        _ => Err(SusumuError::function_call_error(
            "subtract requires 1 or 2 arguments",
        )),
    }
}

fn builtin_multiply(args: &[Value]) -> SusumuResult<Value> {
    if args.is_empty() {
        return Err(SusumuError::function_call_error(
            "multiply requires at least one argument",
        ));
    }

    let mut result = 1.0;
    for arg in args {
        match arg {
            Value::Number(n) => result *= n.as_f64().unwrap(),
            _ => return Err(SusumuError::type_error("number", &format!("{:?}", arg))),
        }
    }
    Ok(create_number_value(result))
}

fn builtin_divide(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::function_call_error(
            "divide requires exactly 2 arguments",
        ));
    }

    match (&args[0], &args[1]) {
        (Value::Number(a), Value::Number(b)) => {
            let b_val = b.as_f64().unwrap();
            if b_val == 0.0 {
                return Err(SusumuError::runtime_error("Division by zero"));
            }
            Ok(create_number_value(a.as_f64().unwrap() / b_val))
        }
        _ => Err(SusumuError::type_error("number", "non-number")),
    }
}

fn builtin_modulo(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::function_call_error(
            "modulo requires exactly 2 arguments",
        ));
    }

    match (&args[0], &args[1]) {
        (Value::Number(a), Value::Number(b)) => {
            let b_val = b.as_f64().unwrap();
            if b_val == 0.0 {
                return Err(SusumuError::runtime_error("Modulo by zero"));
            }
            Ok(create_number_value(a.as_f64().unwrap() % b_val))
        }
        _ => Err(SusumuError::type_error("number", "non-number")),
    }
}

fn builtin_power(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::function_call_error(
            "power requires exactly 2 arguments",
        ));
    }

    match (&args[0], &args[1]) {
        (Value::Number(a), Value::Number(b)) => Ok(create_number_value(
            a.as_f64().unwrap().powf(b.as_f64().unwrap()),
        )),
        _ => Err(SusumuError::type_error("number", "non-number")),
    }
}

fn builtin_sqrt(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "sqrt requires exactly 1 argument",
        ));
    }

    match &args[0] {
        Value::Number(n) => {
            let val = n.as_f64().unwrap();
            if val < 0.0 {
                return Err(SusumuError::runtime_error(
                    "Cannot take square root of negative number",
                ));
            }
            Ok(create_number_value(val.sqrt()))
        }
        _ => Err(SusumuError::type_error("number", &format!("{:?}", args[0]))),
    }
}

fn builtin_abs(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "abs requires exactly 1 argument",
        ));
    }

    match &args[0] {
        Value::Number(n) => Ok(create_number_value(n.as_f64().unwrap().abs())),
        _ => Err(SusumuError::type_error("number", &format!("{:?}", args[0]))),
    }
}

fn builtin_min(args: &[Value]) -> SusumuResult<Value> {
    if args.is_empty() {
        return Err(SusumuError::function_call_error(
            "min requires at least one argument",
        ));
    }

    let mut min_val = f64::INFINITY;
    for arg in args {
        match arg {
            Value::Number(n) => {
                let val = n.as_f64().unwrap();
                if val < min_val {
                    min_val = val;
                }
            }
            _ => return Err(SusumuError::type_error("number", &format!("{:?}", arg))),
        }
    }
    Ok(json!(min_val))
}

fn builtin_max(args: &[Value]) -> SusumuResult<Value> {
    if args.is_empty() {
        return Err(SusumuError::function_call_error(
            "max requires at least one argument",
        ));
    }

    let mut max_val = f64::NEG_INFINITY;
    for arg in args {
        match arg {
            Value::Number(n) => {
                let val = n.as_f64().unwrap();
                if val > max_val {
                    max_val = val;
                }
            }
            _ => return Err(SusumuError::type_error("number", &format!("{:?}", arg))),
        }
    }
    Ok(json!(max_val))
}

fn builtin_sum(args: &[Value]) -> SusumuResult<Value> {
    builtin_add(args)
}

fn builtin_average(args: &[Value]) -> SusumuResult<Value> {
    if args.is_empty() {
        return Err(SusumuError::function_call_error(
            "average requires at least one argument",
        ));
    }

    let sum = builtin_add(args)?;
    match sum {
        Value::Number(s) => Ok(create_number_value(s.as_f64().unwrap() / args.len() as f64)),
        _ => Err(SusumuError::runtime_error("Sum calculation failed")),
    }
}

// String functions
fn builtin_concat(args: &[Value]) -> SusumuResult<Value> {
    let mut result = String::new();
    for arg in args {
        match arg {
            Value::String(s) => result.push_str(s),
            _ => result.push_str(&arg.to_string()),
        }
    }
    Ok(json!(result))
}

fn builtin_length(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "length requires exactly 1 argument",
        ));
    }

    match &args[0] {
        Value::String(s) => Ok(json!(s.len())),
        Value::Array(a) => Ok(json!(a.len())),
        _ => Err(SusumuError::type_error(
            "string or array",
            &format!("{:?}", args[0]),
        )),
    }
}

fn builtin_substring(args: &[Value]) -> SusumuResult<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(SusumuError::function_call_error(
            "substring requires 2 or 3 arguments",
        ));
    }

    match (&args[0], &args[1]) {
        (Value::String(s), Value::Number(start)) => {
            let start_idx = start.as_f64().unwrap() as usize;
            if start_idx > s.len() {
                return Ok(json!(""));
            }

            if args.len() == 3 {
                match &args[2] {
                    Value::Number(end) => {
                        let end_idx = (end.as_f64().unwrap() as usize).min(s.len());
                        if start_idx >= end_idx {
                            return Ok(json!(""));
                        }
                        Ok(json!(s[start_idx..end_idx].to_string()))
                    }
                    _ => Err(SusumuError::type_error("number", &format!("{:?}", args[2]))),
                }
            } else {
                Ok(json!(s[start_idx..].to_string()))
            }
        }
        _ => Err(SusumuError::type_error("string, number", "other types")),
    }
}

fn builtin_to_upper(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "to_upper requires exactly 1 argument",
        ));
    }

    match &args[0] {
        Value::String(s) => Ok(json!(s.to_uppercase())),
        _ => Err(SusumuError::type_error("string", &format!("{:?}", args[0]))),
    }
}

fn builtin_to_lower(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "to_lower requires exactly 1 argument",
        ));
    }

    match &args[0] {
        Value::String(s) => Ok(json!(s.to_lowercase())),
        _ => Err(SusumuError::type_error("string", &format!("{:?}", args[0]))),
    }
}

fn builtin_trim(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "trim requires exactly 1 argument",
        ));
    }

    match &args[0] {
        Value::String(s) => Ok(json!(s.trim())),
        _ => Err(SusumuError::type_error("string", &format!("{:?}", args[0]))),
    }
}

fn builtin_split(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::function_call_error(
            "split requires exactly 2 arguments",
        ));
    }

    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(delimiter)) => {
            let parts: Vec<Value> = s.split(delimiter).map(|part| json!(part)).collect();
            Ok(Value::Array(parts))
        }
        _ => Err(SusumuError::type_error("string, string", "other types")),
    }
}

fn builtin_contains(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::function_call_error(
            "contains requires exactly 2 arguments",
        ));
    }

    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(substr)) => Ok(json!(s.contains(substr))),
        _ => Err(SusumuError::type_error("string, string", "other types")),
    }
}

// Array functions
fn builtin_first(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "first requires exactly 1 argument",
        ));
    }

    match &args[0] {
        Value::Array(a) => {
            if a.is_empty() {
                Ok(Value::Null)
            } else {
                Ok(a[0].clone())
            }
        }
        _ => Err(SusumuError::type_error("array", &format!("{:?}", args[0]))),
    }
}

fn builtin_last(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "last requires exactly 1 argument",
        ));
    }

    match &args[0] {
        Value::Array(a) => {
            if a.is_empty() {
                Ok(Value::Null)
            } else {
                Ok(a[a.len() - 1].clone())
            }
        }
        _ => Err(SusumuError::type_error("array", &format!("{:?}", args[0]))),
    }
}

fn builtin_rest(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "rest requires exactly 1 argument",
        ));
    }

    match &args[0] {
        Value::Array(a) => {
            if a.is_empty() {
                Ok(Value::Array(vec![]))
            } else {
                Ok(Value::Array(a[1..].to_vec()))
            }
        }
        _ => Err(SusumuError::type_error("array", &format!("{:?}", args[0]))),
    }
}

fn builtin_push(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::function_call_error(
            "push requires exactly 2 arguments",
        ));
    }

    match &args[0] {
        Value::Array(a) => {
            let mut new_array = a.clone();
            new_array.push(args[1].clone());
            Ok(Value::Array(new_array))
        }
        _ => Err(SusumuError::type_error("array", &format!("{:?}", args[0]))),
    }
}

fn builtin_pop(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "pop requires exactly 1 argument",
        ));
    }

    match &args[0] {
        Value::Array(a) => {
            if a.is_empty() {
                Ok(Value::Array(vec![]))
            } else {
                let mut new_array = a.clone();
                new_array.pop();
                Ok(Value::Array(new_array))
            }
        }
        _ => Err(SusumuError::type_error("array", &format!("{:?}", args[0]))),
    }
}

fn builtin_filter(_args: &[Value]) -> SusumuResult<Value> {
    // TODO: Implement higher-order function support
    Err(SusumuError::runtime_error("filter not yet implemented"))
}

fn builtin_map(_args: &[Value]) -> SusumuResult<Value> {
    // TODO: Implement higher-order function support
    Err(SusumuError::runtime_error("map not yet implemented"))
}

fn builtin_reduce(_args: &[Value]) -> SusumuResult<Value> {
    // TODO: Implement higher-order function support
    Err(SusumuError::runtime_error("reduce not yet implemented"))
}

fn builtin_sort(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "sort requires exactly 1 argument",
        ));
    }

    match &args[0] {
        Value::Array(a) => {
            let mut sorted = a.clone();
            sorted.sort_by(|a, b| match (a, b) {
                (Value::Number(n1), Value::Number(n2)) => n1
                    .as_f64()
                    .unwrap()
                    .partial_cmp(&n2.as_f64().unwrap())
                    .unwrap(),
                (Value::String(s1), Value::String(s2)) => s1.cmp(s2),
                _ => std::cmp::Ordering::Equal,
            });
            Ok(Value::Array(sorted))
        }
        _ => Err(SusumuError::type_error("array", &format!("{:?}", args[0]))),
    }
}

fn builtin_reverse(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "reverse requires exactly 1 argument",
        ));
    }

    match &args[0] {
        Value::Array(a) => {
            let mut reversed = a.clone();
            reversed.reverse();
            Ok(Value::Array(reversed))
        }
        _ => Err(SusumuError::type_error("array", &format!("{:?}", args[0]))),
    }
}

// I/O functions
fn builtin_print(args: &[Value]) -> SusumuResult<Value> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", value_to_display_string(arg));
    }
    Ok(Value::Null)
}

fn builtin_println(args: &[Value]) -> SusumuResult<Value> {
    builtin_print(args)?;
    println!();
    Ok(Value::Null)
}

fn builtin_debug(args: &[Value]) -> SusumuResult<Value> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{:?}", arg);
    }
    println!();
    Ok(Value::Null)
}

// Utility functions
fn builtin_type_of(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "type_of requires exactly 1 argument",
        ));
    }

    let type_name = match &args[0] {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    };
    Ok(json!(type_name))
}

fn builtin_is_null(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "is_null requires exactly 1 argument",
        ));
    }
    Ok(json!(args[0] == Value::Null))
}

fn builtin_is_number(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "is_number requires exactly 1 argument",
        ));
    }
    Ok(json!(matches!(args[0], Value::Number(_))))
}

fn builtin_is_string(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "is_string requires exactly 1 argument",
        ));
    }
    Ok(json!(matches!(args[0], Value::String(_))))
}

fn builtin_is_boolean(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "is_boolean requires exactly 1 argument",
        ));
    }
    Ok(json!(matches!(args[0], Value::Bool(_))))
}

fn builtin_is_array(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "is_array requires exactly 1 argument",
        ));
    }
    Ok(json!(matches!(args[0], Value::Array(_))))
}

fn builtin_is_object(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "is_object requires exactly 1 argument",
        ));
    }
    Ok(json!(matches!(args[0], Value::Object(_))))
}

fn builtin_to_string(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "to_string requires exactly 1 argument",
        ));
    }
    Ok(json!(value_to_display_string(&args[0])))
}

fn builtin_to_number(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "to_number requires exactly 1 argument",
        ));
    }

    match &args[0] {
        Value::Number(_n) => Ok(args[0].clone()),
        Value::String(s) => match s.parse::<f64>() {
            Ok(n) => Ok(json!(n)),
            Err(_) => Err(SusumuError::type_error("valid number string", s)),
        },
        _ => Err(SusumuError::type_error(
            "number or string",
            &format!("{:?}", args[0]),
        )),
    }
}

/// Convert a JSON value to a human-readable display string
pub fn value_to_display_string(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => {
            if n.is_f64() {
                let f = n.as_f64().unwrap();
                if f.fract() == 0.0 {
                    format!("{}", f as i64)
                } else {
                    f.to_string()
                }
            } else {
                n.to_string()
            }
        }
        Value::String(s) => s.clone(),
        Value::Array(a) => {
            let items: Vec<String> = a.iter().map(value_to_display_string).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Object(o) => {
            let pairs: Vec<String> = o
                .iter()
                .map(|(k, v)| format!("{}: {}", k, value_to_display_string(v)))
                .collect();
            format!("{{{}}}", pairs.join(", "))
        }
    }
}

// Maybe/Result constructor functions
fn builtin_some(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "some requires exactly 1 argument",
        ));
    }

    Ok(json!({
        "type": "some",
        "value": args[0].clone()
    }))
}

fn builtin_none(_args: &[Value]) -> SusumuResult<Value> {
    Ok(json!({
        "type": "none"
    }))
}

fn builtin_success(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "success requires exactly 1 argument",
        ));
    }

    Ok(json!({
        "type": "success",
        "value": args[0].clone()
    }))
}

fn builtin_error(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "error requires exactly 1 argument",
        ));
    }

    Ok(json!({
        "type": "error",
        "value": args[0].clone()
    }))
}

// Maybe/Result type check functions
fn builtin_is_some(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "is_some requires exactly 1 argument",
        ));
    }

    match &args[0] {
        Value::Object(obj) => {
            if let Some(Value::String(type_str)) = obj.get("type") {
                Ok(json!(type_str == "some"))
            } else {
                Ok(json!(false))
            }
        }
        _ => Ok(json!(false)),
    }
}

fn builtin_is_none(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "is_none requires exactly 1 argument",
        ));
    }

    match &args[0] {
        Value::Object(obj) => {
            if let Some(Value::String(type_str)) = obj.get("type") {
                Ok(json!(type_str == "none"))
            } else {
                Ok(json!(false))
            }
        }
        _ => Ok(json!(false)),
    }
}

fn builtin_is_success(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "is_success requires exactly 1 argument",
        ));
    }

    match &args[0] {
        Value::Object(obj) => {
            if let Some(Value::String(type_str)) = obj.get("type") {
                Ok(json!(type_str == "success"))
            } else {
                Ok(json!(false))
            }
        }
        _ => Ok(json!(false)),
    }
}

fn builtin_is_error(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "is_error requires exactly 1 argument",
        ));
    }

    match &args[0] {
        Value::Object(obj) => {
            if let Some(Value::String(type_str)) = obj.get("type") {
                Ok(json!(type_str == "error"))
            } else {
                Ok(json!(false))
            }
        }
        _ => Ok(json!(false)),
    }
}

// Value extraction functions
fn builtin_unwrap(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::function_call_error(
            "unwrap requires exactly 1 argument",
        ));
    }

    match &args[0] {
        Value::Object(obj) => {
            if let Some(Value::String(type_str)) = obj.get("type") {
                match type_str.as_str() {
                    "some" | "success" => {
                        if let Some(value) = obj.get("value") {
                            Ok(value.clone())
                        } else {
                            Err(SusumuError::runtime_error("Cannot unwrap: missing value"))
                        }
                    }
                    "none" => Err(SusumuError::runtime_error("Cannot unwrap none")),
                    "error" => Err(SusumuError::runtime_error("Cannot unwrap error")),
                    _ => Err(SusumuError::runtime_error(
                        "Cannot unwrap: not a Maybe or Result type",
                    )),
                }
            } else {
                Err(SusumuError::runtime_error(
                    "Cannot unwrap: not a Maybe or Result type",
                ))
            }
        }
        _ => Err(SusumuError::runtime_error(
            "Cannot unwrap: not a Maybe or Result type",
        )),
    }
}

fn builtin_unwrap_or(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::function_call_error(
            "unwrap_or requires exactly 2 arguments",
        ));
    }

    match &args[0] {
        Value::Object(obj) => {
            if let Some(Value::String(type_str)) = obj.get("type") {
                match type_str.as_str() {
                    "some" | "success" => {
                        if let Some(value) = obj.get("value") {
                            Ok(value.clone())
                        } else {
                            Ok(args[1].clone()) // fallback
                        }
                    }
                    "none" | "error" => Ok(args[1].clone()), // Return default value
                    _ => Ok(args[1].clone()),                // Not Maybe/Result, return default
                }
            } else {
                Ok(args[1].clone()) // Not Maybe/Result, return default
            }
        }
        _ => Ok(args[1].clone()), // Not Maybe/Result, return default
    }
}

/// Combine multiple values by concatenating them as strings (useful for debugging convergent flows)
fn builtin_combine(args: &[Value]) -> SusumuResult<Value> {
    if args.is_empty() {
        return Ok(json!(""));
    }

    let combined = args
        .iter()
        .map(|v| match v {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Array(a) => format!("[{} items]", a.len()),
            Value::Object(o) => format!("{{object with {} fields}}", o.len()),
        })
        .collect::<Vec<String>>()
        .join("");

    Ok(json!(combined))
}

impl BuiltinRegistry {
    /// Register module system functions
    fn register_module_functions(&mut self) {
        self.register("from", builtin_from);
        self.register("import", builtin_import);
        self.register("export", builtin_export);
    }
}

/// Module function: from(module_name, import_spec) -> module_import
/// Note: This function creates an import specification, actual loading happens in import()
fn builtin_from(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "from() expects exactly 1 argument: module_name",
        ));
    }

    let module_name = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(SusumuError::runtime_error("Module name must be a string")),
    };

    // Validate module_name format
    if module_name.is_empty() || module_name.contains("..") || module_name.contains('/') {
        return Err(SusumuError::runtime_error(
            "Invalid module name. Use simple names like 'test_module'",
        ));
    }

    // Create a module reference that import() can use
    Ok(json!({
        "type": "module_reference",
        "module_name": module_name
    }))
}

/// Module function: import(module_import, function_list) -> imported_functions
fn builtin_import(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::runtime_error(
            "import() expects exactly 2 arguments: module_import and function_list",
        ));
    }

    let module_import = &args[0];
    let function_list = &args[1];

    // Validate module_import structure
    let module_name = match module_import {
        Value::Object(obj) if obj.get("type").and_then(|v| v.as_str()) == Some("module_import") => {
            obj.get("module")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
        }
        _ => {
            return Err(SusumuError::runtime_error(
                "First argument must be a module import object",
            ))
        }
    };

    // Parse function list
    let functions = match function_list {
        Value::Array(arr) => arr
            .iter()
            .map(|v| v.as_str().unwrap_or("unknown").to_string())
            .collect::<Vec<_>>(),
        Value::String(s) => vec![s.clone()],
        _ => {
            return Err(SusumuError::runtime_error(
                "Function list must be array or string",
            ))
        }
    };

    // In a real implementation, this would:
    // 1. Resolve the module file system path
    // 2. Load and parse the module
    // 3. Validate exported functions exist
    // 4. Return function references or inject them into current scope

    // For now, return a placeholder that shows what was imported
    Ok(json!({
        "type": "imported_functions",
        "module": module_name,
        "functions": functions,
        "status": "placeholder - not yet implemented"
    }))
}

/// Module function: export(function_list) -> export_declaration  
fn builtin_export(args: &[Value]) -> SusumuResult<Value> {
    if args.is_empty() {
        return Err(SusumuError::runtime_error(
            "export() expects at least 1 argument",
        ));
    }

    let exported_functions = if args.len() == 1 {
        match &args[0] {
            Value::Array(arr) => arr
                .iter()
                .map(|v| v.as_str().unwrap_or("unknown").to_string())
                .collect::<Vec<_>>(),
            Value::String(s) => vec![s.clone()],
            _ => {
                return Err(SusumuError::runtime_error(
                    "Export argument must be function name or array of names",
                ))
            }
        }
    } else {
        args.iter()
            .map(|v| v.as_str().unwrap_or("unknown").to_string())
            .collect()
    };

    // In a real implementation, this would:
    // 1. Validate all functions exist in current scope
    // 2. Mark them as exported in module metadata
    // 3. Make them available for import by other modules

    Ok(json!({
        "type": "export_declaration",
        "functions": exported_functions,
        "status": "placeholder - not yet implemented"
    }))
}

// =============================================================================
// CORE AUTO-AVAILABLE FUNCTIONS (Hybrid Approach)
// These functions are immediately available without imports
// =============================================================================

/// Core add function with overflow protection - supports multiple arguments for convergence
fn builtin_core_add(args: &[Value]) -> SusumuResult<Value> {
    if args.is_empty() {
        return Err(SusumuError::runtime_error(
            "add() expects at least 1 argument",
        ));
    }

    let mut result = 0.0;
    for (i, arg) in args.iter().enumerate() {
        let num = arg
            .as_f64()
            .ok_or_else(|| SusumuError::runtime_error(&format!("Argument {} must be a number", i + 1)))?;
        result += num;
    }

    if result.is_finite() {
        Ok(json!(result))
    } else {
        Err(SusumuError::runtime_error(
            "Arithmetic overflow in addition",
        ))
    }
}

/// Core subtract function
fn builtin_core_subtract(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::runtime_error(
            "subtract() expects exactly 2 arguments",
        ));
    }

    let a = args[0]
        .as_f64()
        .ok_or_else(|| SusumuError::runtime_error("First argument must be a number"))?;
    let b = args[1]
        .as_f64()
        .ok_or_else(|| SusumuError::runtime_error("Second argument must be a number"))?;

    let result = a - b;
    if result.is_finite() {
        Ok(json!(result))
    } else {
        Err(SusumuError::runtime_error(
            "Arithmetic underflow in subtraction",
        ))
    }
}

/// Core multiply function  
fn builtin_core_multiply(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::runtime_error(
            "multiply() expects exactly 2 arguments",
        ));
    }

    let a = args[0]
        .as_f64()
        .ok_or_else(|| SusumuError::runtime_error("First argument must be a number"))?;
    let b = args[1]
        .as_f64()
        .ok_or_else(|| SusumuError::runtime_error("Second argument must be a number"))?;

    let result = a * b;
    if result.is_finite() {
        Ok(json!(result))
    } else {
        Err(SusumuError::runtime_error(
            "Arithmetic overflow in multiplication",
        ))
    }
}

/// Core divide function with division by zero protection
fn builtin_core_divide(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::runtime_error(
            "divide() expects exactly 2 arguments",
        ));
    }

    let a = args[0]
        .as_f64()
        .ok_or_else(|| SusumuError::runtime_error("First argument must be a number"))?;
    let b = args[1]
        .as_f64()
        .ok_or_else(|| SusumuError::runtime_error("Second argument must be a number"))?;

    if b == 0.0 {
        return Err(SusumuError::runtime_error("Division by zero"));
    }

    let result = a / b;
    if result.is_finite() {
        Ok(json!(result))
    } else {
        Err(SusumuError::runtime_error(
            "Arithmetic overflow in division",
        ))
    }
}

/// Core print function with automatic newline and memory-efficient output
fn builtin_core_print(args: &[Value]) -> SusumuResult<Value> {
    use std::io::{self, Write};

    if args.is_empty() {
        println!();
        return Ok(json!(null));
    }

    // Use stdout directly for better performance and memory management
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            write!(handle, " ")
                .map_err(|e| SusumuError::io_error(format!("Print error: {}", e)))?;
        }
        write!(handle, "{}", value_to_display_string(arg))
            .map_err(|e| SusumuError::io_error(format!("Print error: {}", e)))?;
    }
    writeln!(handle).map_err(|e| SusumuError::io_error(format!("Print error: {}", e)))?;

    // Explicit flush to ensure output is written immediately
    handle
        .flush()
        .map_err(|e| SusumuError::io_error(format!("Print flush error: {}", e)))?;

    Ok(json!(null))
}

/// Core println function (alias for print with explicit newline semantics)
fn builtin_core_println(args: &[Value]) -> SusumuResult<Value> {
    builtin_core_print(args)
}

/// Core toString function with comprehensive type support and memory efficiency
fn builtin_core_to_string(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "toString() expects exactly 1 argument",
        ));
    }

    let result = match &args[0] {
        Value::String(s) => s.clone(),
        Value::Number(n) => {
            // serde_json::Number handles integer/float formatting automatically
            n.to_string()
        }
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else {
                let elements: Result<Vec<String>, SusumuError> = arr
                    .iter()
                    .map(|v| {
                        builtin_core_to_string(&[v.clone()]).and_then(|json_val| {
                            json_val.as_str().map(|s| s.to_string()).ok_or_else(|| {
                                SusumuError::runtime_error("Internal toString error")
                            })
                        })
                    })
                    .collect();

                match elements {
                    Ok(strs) => format!("[{}]", strs.join(", ")),
                    Err(_) => "[?]".to_string(),
                }
            }
        }
        Value::Object(obj) => {
            if obj.is_empty() {
                "{}".to_string()
            } else {
                let pairs: Result<Vec<String>, SusumuError> = obj
                    .iter()
                    .map(|(k, v)| {
                        builtin_core_to_string(&[v.clone()]).and_then(|json_val| {
                            json_val
                                .as_str()
                                .map(|s| format!("{}: {}", k, s))
                                .ok_or_else(|| {
                                    SusumuError::runtime_error("Internal toString error")
                                })
                        })
                    })
                    .collect();

                match pairs {
                    Ok(strs) => format!("{{{}}}", strs.join(", ")),
                    Err(_) => "{?}".to_string(),
                }
            }
        }
    };

    Ok(json!(result))
}

/// Core toNumber function with validation and edge case handling
fn builtin_core_to_number(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "toNumber() expects exactly 1 argument",
        ));
    }

    match &args[0] {
        Value::Number(n) => Ok(json!(n)),
        Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                return Ok(json!(0.0));
            }

            trimmed
                .parse::<f64>()
                .map(|n| {
                    if n.is_finite() {
                        json!(n)
                    } else {
                        json!(null) // Return null for invalid numbers rather than error
                    }
                })
                .map_err(|_| {
                    SusumuError::runtime_error(format!("Cannot convert '{}' to number", s))
                })
        }
        Value::Bool(b) => Ok(json!(if *b { 1.0 } else { 0.0 })),
        Value::Null => Ok(json!(0.0)),
        Value::Array(arr) => Ok(json!(arr.len() as f64)),
        Value::Object(obj) => Ok(json!(obj.len() as f64)),
    }
}

/// Core validation functions with memory efficiency
fn builtin_core_is_null(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "isNull() expects exactly 1 argument",
        ));
    }
    Ok(json!(args[0].is_null()))
}

fn builtin_core_is_empty(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "isEmpty() expects exactly 1 argument",
        ));
    }

    let is_empty = match &args[0] {
        Value::Null => true,
        Value::String(s) => s.is_empty(),
        Value::Array(arr) => arr.is_empty(),
        Value::Object(obj) => obj.is_empty(),
        Value::Number(n) => n.as_f64().unwrap_or(1.0) == 0.0,
        Value::Bool(b) => !b,
    };

    Ok(json!(is_empty))
}

fn builtin_core_is_number(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "isNumber() expects exactly 1 argument",
        ));
    }
    Ok(json!(args[0].is_number()))
}

fn builtin_core_is_string(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "isString() expects exactly 1 argument",
        ));
    }
    Ok(json!(args[0].is_string()))
}

fn builtin_core_length(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "length() expects exactly 1 argument",
        ));
    }

    let len = match &args[0] {
        Value::String(s) => s.chars().count() as f64, // UTF-8 safe character count
        Value::Array(arr) => arr.len() as f64,
        Value::Object(obj) => obj.len() as f64,
        _ => {
            return Err(SusumuError::runtime_error(
                "length() only works with strings, arrays, or objects",
            ))
        }
    };

    Ok(json!(len))
}

fn builtin_core_type(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "type() expects exactly 1 argument",
        ));
    }

    let type_name = match &args[0] {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    };

    Ok(json!(type_name))
}

fn builtin_core_equals(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::runtime_error(
            "equals() expects exactly 2 arguments",
        ));
    }

    // Deep equality comparison with JSON values
    Ok(json!(args[0] == args[1]))
}

// =============================================================================
// COMPREHENSIVE I/O FUNCTIONS (Auto-available for productivity)
// Memory-efficient with automatic resource management
// =============================================================================

/// Read entire file with automatic resource cleanup
fn builtin_core_read_file(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "readFile() expects exactly 1 argument (file path)",
        ));
    }

    let file_path = args[0]
        .as_str()
        .ok_or_else(|| SusumuError::runtime_error("readFile() expects a string file path"))?;

    // Automatic resource management - file handle is dropped at end of scope
    match std::fs::read_to_string(file_path) {
        Ok(content) => Ok(json!(content)),
        Err(e) => Err(SusumuError::io_error(format!(
            "Failed to read file '{}': {}",
            file_path, e
        ))),
    }
}

/// Write file with automatic resource cleanup and UTF-8 safety  
fn builtin_core_write_file(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::runtime_error(
            "writeFile() expects exactly 2 arguments (file path, content)",
        ));
    }

    let file_path = args[0]
        .as_str()
        .ok_or_else(|| SusumuError::runtime_error("writeFile() expects a string file path"))?;
    let content = args[1]
        .as_str()
        .ok_or_else(|| SusumuError::runtime_error("writeFile() expects string content"))?;

    // Automatic resource management with explicit error handling
    match std::fs::write(file_path, content) {
        Ok(()) => Ok(json!({"success": true, "bytes_written": content.len()})),
        Err(e) => Err(SusumuError::io_error(format!(
            "Failed to write file '{}': {}",
            file_path, e
        ))),
    }
}

/// Append to file with automatic resource cleanup
fn builtin_core_append_file(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::runtime_error(
            "appendFile() expects exactly 2 arguments (file path, content)",
        ));
    }

    let file_path = args[0]
        .as_str()
        .ok_or_else(|| SusumuError::runtime_error("appendFile() expects a string file path"))?;
    let content = args[1]
        .as_str()
        .ok_or_else(|| SusumuError::runtime_error("appendFile() expects string content"))?;

    use std::fs::OpenOptions;
    use std::io::Write;

    // Memory-efficient append with automatic resource cleanup
    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
        .and_then(|mut file| {
            file.write_all(content.as_bytes())?;
            file.flush()?; // Ensure data is written
            Ok(())
        }) {
        Ok(()) => Ok(json!({"success": true, "bytes_appended": content.len()})),
        Err(e) => Err(SusumuError::io_error(format!(
            "Failed to append to file '{}': {}",
            file_path, e
        ))),
    }
}

/// Check if file exists
fn builtin_core_file_exists(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "fileExists() expects exactly 1 argument (file path)",
        ));
    }

    let file_path = args[0]
        .as_str()
        .ok_or_else(|| SusumuError::runtime_error("fileExists() expects a string file path"))?;

    Ok(json!(std::path::Path::new(file_path).exists()))
}

/// Get file metadata with automatic resource management
fn builtin_core_file_info(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "fileInfo() expects exactly 1 argument (file path)",
        ));
    }

    let file_path = args[0]
        .as_str()
        .ok_or_else(|| SusumuError::runtime_error("fileInfo() expects a string file path"))?;

    match std::fs::metadata(file_path) {
        Ok(metadata) => Ok(json!({
            "size": metadata.len(),
            "is_file": metadata.is_file(),
            "is_directory": metadata.is_dir(),
            "readonly": metadata.permissions().readonly(),
            "modified": metadata.modified()
                .ok()
                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|dur| dur.as_secs())
                .unwrap_or(0)
        })),
        Err(e) => Err(SusumuError::io_error(format!(
            "Failed to get file info for '{}': {}",
            file_path, e
        ))),
    }
}

/// List directory contents
fn builtin_core_list_dir(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "listDir() expects exactly 1 argument (directory path)",
        ));
    }

    let dir_path = args[0]
        .as_str()
        .ok_or_else(|| SusumuError::runtime_error("listDir() expects a string directory path"))?;

    match std::fs::read_dir(dir_path) {
        Ok(entries) => {
            let mut file_list = Vec::new();
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        if let Some(name) = entry.file_name().to_str() {
                            file_list.push(json!(name));
                        }
                    }
                    Err(e) => {
                        return Err(SusumuError::io_error(format!(
                            "Error reading directory entry: {}",
                            e
                        )))
                    }
                }
            }
            Ok(json!(file_list))
        }
        Err(e) => Err(SusumuError::io_error(format!(
            "Failed to list directory '{}': {}",
            dir_path, e
        ))),
    }
}

// =============================================================================
// JSON/DATA PROCESSING FUNCTIONS (Auto-available)
// =============================================================================

/// Parse JSON string with comprehensive error handling
fn builtin_core_parse_json(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "parseJSON() expects exactly 1 argument (JSON string)",
        ));
    }

    let json_str = args[0]
        .as_str()
        .ok_or_else(|| SusumuError::runtime_error("parseJSON() expects a string"))?;

    match serde_json::from_str::<Value>(json_str) {
        Ok(parsed) => Ok(parsed),
        Err(e) => Err(SusumuError::runtime_error(format!("Invalid JSON: {}", e))),
    }
}

/// Convert value to JSON string with pretty formatting option
fn builtin_core_to_json(args: &[Value]) -> SusumuResult<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(SusumuError::runtime_error(
            "toJSON() expects 1-2 arguments (value, pretty?)",
        ));
    }

    let pretty = args.get(1).and_then(|v| v.as_bool()).unwrap_or(false);

    let json_str = if pretty {
        serde_json::to_string_pretty(&args[0])
    } else {
        serde_json::to_string(&args[0])
    };

    match json_str {
        Ok(json) => Ok(json!(json)),
        Err(e) => Err(SusumuError::runtime_error(format!(
            "Failed to serialize to JSON: {}",
            e
        ))),
    }
}

// =============================================================================
// ARRAY PROCESSING FUNCTIONS (Auto-available)
// =============================================================================

/// Filter array with predicate function
fn builtin_core_filter(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::runtime_error(
            "filter() expects exactly 2 arguments (array, predicate)",
        ));
    }

    let array = args[0]
        .as_array()
        .ok_or_else(|| SusumuError::runtime_error("filter() expects an array as first argument"))?;

    // For now, simple value-based filtering (in future: function predicates)
    let filter_value = &args[1];

    let filtered: Vec<Value> = array
        .iter()
        .filter(|&item| item == filter_value)
        .cloned()
        .collect();

    Ok(json!(filtered))
}

/// Map array with transformation function
fn builtin_core_map(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::runtime_error(
            "map() expects exactly 2 arguments (array, transform)",
        ));
    }

    let array = args[0]
        .as_array()
        .ok_or_else(|| SusumuError::runtime_error("map() expects an array as first argument"))?;

    // For now, simple transformation (in future: function application)
    // This is a placeholder - real implementation would apply transform function
    Ok(json!(array.clone()))
}

/// Reduce array to single value
fn builtin_core_reduce(args: &[Value]) -> SusumuResult<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(SusumuError::runtime_error(
            "reduce() expects 2-3 arguments (array, reducer, initial?)",
        ));
    }

    let array = args[0]
        .as_array()
        .ok_or_else(|| SusumuError::runtime_error("reduce() expects an array as first argument"))?;

    if array.is_empty() {
        return Ok(args.get(2).cloned().unwrap_or(json!(null)));
    }

    // For now, simple sum operation (in future: custom reducer functions)
    let sum: f64 = array.iter().filter_map(|v| v.as_f64()).sum();

    Ok(json!(sum))
}

// =============================================================================
// DATE/TIME FUNCTIONS (Auto-available for productivity)
// =============================================================================

/// Get current timestamp (seconds since Unix epoch)
fn builtin_core_now(args: &[Value]) -> SusumuResult<Value> {
    if !args.is_empty() {
        return Err(SusumuError::runtime_error("now() expects no arguments"));
    }

    use std::time::{SystemTime, UNIX_EPOCH};

    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => Ok(json!(duration.as_secs())),
        Err(e) => Err(SusumuError::runtime_error(format!(
            "Failed to get current time: {}",
            e
        ))),
    }
}

/// Get current timestamp in milliseconds
fn builtin_core_now_millis(args: &[Value]) -> SusumuResult<Value> {
    if !args.is_empty() {
        return Err(SusumuError::runtime_error(
            "nowMillis() expects no arguments",
        ));
    }

    use std::time::{SystemTime, UNIX_EPOCH};

    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => Ok(json!(duration.as_millis() as u64)),
        Err(e) => Err(SusumuError::runtime_error(format!(
            "Failed to get current time: {}",
            e
        ))),
    }
}

/// Format timestamp as ISO 8601 string (UTC)
fn builtin_core_format_date(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "formatDate() expects exactly 1 argument (timestamp)",
        ));
    }

    let timestamp = args[0]
        .as_f64()
        .ok_or_else(|| SusumuError::runtime_error("formatDate() expects a number timestamp"))?;

    use chrono::{TimeZone, Utc};

    match Utc.timestamp_opt(timestamp as i64, 0) {
        chrono::LocalResult::Single(datetime) => Ok(json!(datetime.to_rfc3339())),
        _ => Err(SusumuError::runtime_error("Invalid timestamp")),
    }
}

/// Parse ISO 8601 date string to timestamp
fn builtin_core_parse_date(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "parseDate() expects exactly 1 argument (ISO date string)",
        ));
    }

    let date_str = args[0]
        .as_str()
        .ok_or_else(|| SusumuError::runtime_error("parseDate() expects a string"))?;

    use chrono::{DateTime, Utc};

    match DateTime::parse_from_rfc3339(date_str) {
        Ok(datetime) => Ok(json!(datetime.timestamp())),
        Err(_) => {
            // Try parsing as ISO 8601 with Z suffix
            match date_str.parse::<DateTime<Utc>>() {
                Ok(datetime) => Ok(json!(datetime.timestamp())),
                Err(e) => Err(SusumuError::runtime_error(format!(
                    "Invalid date format '{}': {}",
                    date_str, e
                ))),
            }
        }
    }
}

/// Add duration to timestamp (in seconds)
fn builtin_core_add_time(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::runtime_error(
            "addTime() expects exactly 2 arguments (timestamp, seconds)",
        ));
    }

    let timestamp = args[0]
        .as_f64()
        .ok_or_else(|| SusumuError::runtime_error("addTime() expects a number timestamp"))?;
    let seconds = args[1]
        .as_f64()
        .ok_or_else(|| SusumuError::runtime_error("addTime() expects a number of seconds"))?;

    Ok(json!(timestamp + seconds))
}

/// Calculate difference between two timestamps (in seconds)
fn builtin_core_time_diff(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::runtime_error(
            "timeDiff() expects exactly 2 arguments (timestamp1, timestamp2)",
        ));
    }

    let timestamp1 = args[0]
        .as_f64()
        .ok_or_else(|| SusumuError::runtime_error("timeDiff() expects number timestamps"))?;
    let timestamp2 = args[1]
        .as_f64()
        .ok_or_else(|| SusumuError::runtime_error("timeDiff() expects number timestamps"))?;

    Ok(json!((timestamp2 - timestamp1).abs()))
}

// === STDLIB IMPLEMENTATION FUNCTIONS ===

fn builtin_calculate_factorial(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "calculate_factorial() expects exactly 1 argument",
        ));
    }

    let n = args[0]
        .as_f64()
        .ok_or_else(|| SusumuError::runtime_error("calculate_factorial() expects a number"))?;

    if n < 0.0 {
        return Err(SusumuError::runtime_error(
            "Factorial not defined for negative numbers",
        ));
    }

    let n = n as u64;
    let mut result = 1u64;
    for i in 1..=n {
        result *= i;
    }

    Ok(json!(result))
}

fn builtin_calculate_gcd(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "calculate_gcd() expects exactly 1 argument (tuple)",
        ));
    }

    let tuple = args[0]
        .as_array()
        .ok_or_else(|| SusumuError::runtime_error("calculate_gcd() expects a tuple/array"))?;

    if tuple.len() != 2 {
        return Err(SusumuError::runtime_error(
            "calculate_gcd() expects a tuple with exactly 2 numbers",
        ));
    }

    let a = tuple[0]
        .as_f64()
        .ok_or_else(|| SusumuError::runtime_error("calculate_gcd() expects numbers"))?
        as i64;
    let b = tuple[1]
        .as_f64()
        .ok_or_else(|| SusumuError::runtime_error("calculate_gcd() expects numbers"))?
        as i64;

    fn gcd(a: i64, b: i64) -> i64 {
        if b == 0 {
            a.abs()
        } else {
            gcd(b, a % b)
        }
    }

    Ok(json!(gcd(a, b)))
}

fn builtin_check_prime_factors(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "check_prime_factors() expects exactly 1 argument",
        ));
    }

    let n = args[0]
        .as_f64()
        .ok_or_else(|| SusumuError::runtime_error("check_prime_factors() expects a number"))?
        as i64;

    if n <= 1 {
        return Ok(json!(false));
    }

    for i in 2..=((n as f64).sqrt() as i64) {
        if n % i == 0 {
            return Ok(json!(false));
        }
    }

    Ok(json!(true))
}

fn builtin_calculate_fibonacci(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "calculate_fibonacci() expects exactly 1 argument",
        ));
    }

    let n = args[0]
        .as_f64()
        .ok_or_else(|| SusumuError::runtime_error("calculate_fibonacci() expects a number"))?
        as u64;

    if n <= 1 {
        return Ok(json!(n));
    }

    let mut a = 0u64;
    let mut b = 1u64;
    for _ in 2..=n {
        let temp = a + b;
        a = b;
        b = temp;
    }

    Ok(json!(b))
}

fn builtin_split_string(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::runtime_error(
            "split_string() expects exactly 2 arguments (text, delimiter)",
        ));
    }

    let text = args[0].as_str().ok_or_else(|| {
        SusumuError::runtime_error("split_string() expects a string as first argument")
    })?;
    let delimiter = args[1].as_str().ok_or_else(|| {
        SusumuError::runtime_error("split_string() expects a string delimiter as second argument")
    })?;

    let parts: Vec<Value> = text.split(delimiter).map(|s| json!(s)).collect();

    Ok(json!(parts))
}

fn builtin_join_array(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::runtime_error(
            "join_array() expects exactly 2 arguments (array, delimiter)",
        ));
    }

    let array = args[0].as_array().ok_or_else(|| {
        SusumuError::runtime_error("join_array() expects an array as first argument")
    })?;
    let delimiter = args[1].as_str().ok_or_else(|| {
        SusumuError::runtime_error("join_array() expects a string delimiter as second argument")
    })?;

    let string_parts: Vec<String> = array
        .iter()
        .map(|v| v.as_str().unwrap_or("").to_string())
        .collect();

    Ok(json!(string_parts.join(delimiter)))
}

fn builtin_to_title_case(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "to_title_case() expects exactly 1 argument",
        ));
    }

    let text = args[0]
        .as_str()
        .ok_or_else(|| SusumuError::runtime_error("to_title_case() expects a string"))?;

    let title_case = text
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    Ok(json!(title_case))
}

fn builtin_core_abs(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "abs() expects exactly 1 argument",
        ));
    }

    let n = args[0]
        .as_f64()
        .ok_or_else(|| SusumuError::runtime_error("abs() expects a number"))?;

    Ok(json!(n.abs()))
}

fn builtin_core_first(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "first() expects exactly 1 argument",
        ));
    }

    let array = args[0]
        .as_array()
        .ok_or_else(|| SusumuError::runtime_error("first() expects an array"))?;

    if array.is_empty() {
        Ok(json!(null))
    } else {
        Ok(array[0].clone())
    }
}

fn builtin_core_last(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "last() expects exactly 1 argument",
        ));
    }

    let array = args[0]
        .as_array()
        .ok_or_else(|| SusumuError::runtime_error("last() expects an array"))?;

    if array.is_empty() {
        Ok(json!(null))
    } else {
        Ok(array[array.len() - 1].clone())
    }
}

fn builtin_core_reverse(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "reverse() expects exactly 1 argument",
        ));
    }

    let array = args[0]
        .as_array()
        .ok_or_else(|| SusumuError::runtime_error("reverse() expects an array"))?;

    let mut reversed = array.clone();
    reversed.reverse();
    Ok(json!(reversed))
}

fn builtin_core_sum(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "sum() expects exactly 1 argument",
        ));
    }

    let array = args[0]
        .as_array()
        .ok_or_else(|| SusumuError::runtime_error("sum() expects an array"))?;

    let mut total = 0.0;
    for item in array {
        if let Some(num) = item.as_f64() {
            total += num;
        } else {
            return Err(SusumuError::runtime_error("sum() expects array of numbers"));
        }
    }

    Ok(json!(total))
}

fn builtin_core_is_array(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "isArray() expects exactly 1 argument",
        ));
    }

    Ok(json!(args[0].is_array()))
}

// =============================================================================
// HTTP CLIENT FUNCTIONS (Auto-available with connection pooling)
// =============================================================================

/// Simple HTTP GET request with automatic connection management
fn builtin_core_http_get(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "httpGet() expects exactly 1 argument (URL)",
        ));
    }

    let url = args[0]
        .as_str()
        .ok_or_else(|| SusumuError::runtime_error("httpGet() expects a string URL"))?;

    match reqwest::blocking::get(url) {
        Ok(response) => {
            let status = response.status().as_u16();
            let headers = response
                .headers()
                .iter()
                .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("").to_string()))
                .collect::<std::collections::HashMap<String, String>>();

            match response.text() {
                Ok(body) => Ok(json!({
                    "status": status,
                    "body": body,
                    "headers": headers
                })),
                Err(e) => Err(SusumuError::io_error(format!(
                    "Failed to read response body: {}",
                    e
                ))),
            }
        }
        Err(e) => Err(SusumuError::io_error(format!(
            "HTTP GET failed for '{}': {}",
            url, e
        ))),
    }
}

/// Simple HTTP POST request with JSON payload
fn builtin_core_http_post(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::runtime_error(
            "httpPost() expects exactly 2 arguments (URL, data)",
        ));
    }

    let url = args[0]
        .as_str()
        .ok_or_else(|| SusumuError::runtime_error("httpPost() expects a string URL"))?;

    let client = reqwest::blocking::Client::new();

    match client.post(url).json(&args[1]).send() {
        Ok(response) => {
            let status = response.status().as_u16();
            let headers = response
                .headers()
                .iter()
                .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("").to_string()))
                .collect::<std::collections::HashMap<String, String>>();

            match response.text() {
                Ok(body) => Ok(json!({
                    "status": status,
                    "body": body,
                    "headers": headers
                })),
                Err(e) => Err(SusumuError::io_error(format!(
                    "Failed to read response body: {}",
                    e
                ))),
            }
        }
        Err(e) => Err(SusumuError::io_error(format!(
            "HTTP POST failed for '{}': {}",
            url, e
        ))),
    }
}

/// HTTP request with full configuration
fn builtin_core_http_request(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "httpRequest() expects exactly 1 argument (config object)",
        ));
    }

    let config = args[0].as_object().ok_or_else(|| {
        SusumuError::runtime_error("httpRequest() expects a configuration object")
    })?;

    let url = config.get("url").and_then(|v| v.as_str()).ok_or_else(|| {
        SusumuError::runtime_error("httpRequest() config must include 'url' field")
    })?;

    let method = config
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("GET");

    let client = reqwest::blocking::Client::new();
    let mut request_builder = match method.to_uppercase().as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        "PATCH" => client.patch(url),
        _ => {
            return Err(SusumuError::runtime_error(format!(
                "Unsupported HTTP method: {}",
                method
            )))
        }
    };

    // Add headers if provided
    if let Some(headers_obj) = config.get("headers").and_then(|v| v.as_object()) {
        for (key, value) in headers_obj {
            if let Some(value_str) = value.as_str() {
                request_builder = request_builder.header(key, value_str);
            }
        }
    }

    // Add body if provided
    if let Some(body) = config.get("body") {
        request_builder = request_builder.json(body);
    }

    // Add timeout if provided
    if let Some(timeout_secs) = config.get("timeout").and_then(|v| v.as_f64()) {
        let timeout = std::time::Duration::from_secs(timeout_secs as u64);
        request_builder = request_builder.timeout(timeout);
    }

    match request_builder.send() {
        Ok(response) => {
            let status = response.status().as_u16();
            let headers = response
                .headers()
                .iter()
                .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("").to_string()))
                .collect::<std::collections::HashMap<String, String>>();

            match response.text() {
                Ok(body) => Ok(json!({
                    "status": status,
                    "body": body,
                    "headers": headers
                })),
                Err(e) => Err(SusumuError::io_error(format!(
                    "Failed to read response body: {}",
                    e
                ))),
            }
        }
        Err(e) => Err(SusumuError::io_error(format!(
            "HTTP {} request failed for '{}': {}",
            method, url, e
        ))),
    }
}

/// Parallel HTTP requests - takes array of URLs and fetches them concurrently
fn builtin_core_http_get_parallel(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "httpGetParallel() expects exactly 1 argument (array of URLs)",
        ));
    }

    let urls = args[0]
        .as_array()
        .ok_or_else(|| SusumuError::runtime_error("httpGetParallel() expects an array of URLs"))?;

    // Extract URLs as strings
    let url_strings: Result<Vec<&str>, _> = urls
        .iter()
        .map(|v| {
            v.as_str()
                .ok_or_else(|| SusumuError::runtime_error("All URLs must be strings"))
        })
        .collect();
    let url_strings = url_strings?;

    // Use rayon for parallel blocking requests
    #[cfg(feature = "parallel")]
    {
        use rayon::prelude::*;

        let results: Vec<Value> = url_strings
            .par_iter()
            .map(|&url| match reqwest::blocking::get(url) {
                Ok(response) => {
                    let status = response.status().as_u16();
                    let headers = response
                        .headers()
                        .iter()
                        .map(|(name, value)| {
                            (name.to_string(), value.to_str().unwrap_or("").to_string())
                        })
                        .collect::<std::collections::HashMap<String, String>>();

                    match response.text() {
                        Ok(body) => json!({
                            "url": url,
                            "status": status,
                            "body": body,
                            "headers": headers,
                            "success": true
                        }),
                        Err(e) => json!({
                            "url": url,
                            "error": format!("Failed to read response: {}", e),
                            "success": false
                        }),
                    }
                }
                Err(e) => json!({
                    "url": url,
                    "error": format!("Request failed: {}", e),
                    "success": false
                }),
            })
            .collect();

        Ok(json!(results))
    }

    #[cfg(not(feature = "parallel"))]
    {
        // Fallback to sequential requests if parallel feature not enabled
        let mut results = Vec::new();
        for &url in &url_strings {
            let result = match reqwest::blocking::get(url) {
                Ok(response) => {
                    let status = response.status().as_u16();
                    let headers = response
                        .headers()
                        .iter()
                        .map(|(name, value)| {
                            (name.to_string(), value.to_str().unwrap_or("").to_string())
                        })
                        .collect::<std::collections::HashMap<String, String>>();

                    match response.text() {
                        Ok(body) => json!({
                            "url": url,
                            "status": status,
                            "body": body,
                            "headers": headers,
                            "success": true
                        }),
                        Err(e) => json!({
                            "url": url,
                            "error": format!("Failed to read response: {}", e),
                            "success": false
                        }),
                    }
                }
                Err(e) => json!({
                    "url": url,
                    "error": format!("Request failed: {}", e),
                    "success": false
                }),
            };
            results.push(result);
        }
        Ok(json!(results))
    }
}

/// Parallel HTTP POST requests - takes array of request configs
fn builtin_core_http_post_parallel(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "httpPostParallel() expects exactly 1 argument (array of request configs)",
        ));
    }

    let configs = args[0].as_array().ok_or_else(|| {
        SusumuError::runtime_error("httpPostParallel() expects an array of request configs")
    })?;

    #[cfg(feature = "parallel")]
    {
        use rayon::prelude::*;

        let results: Vec<Value> = configs
            .par_iter()
            .map(|config| {
                let config_obj = match config.as_object() {
                    Some(obj) => obj,
                    None => {
                        return json!({
                            "error": "Each config must be an object with 'url' and 'data' fields",
                            "success": false
                        })
                    }
                };

                let url = match config_obj.get("url").and_then(|v| v.as_str()) {
                    Some(url) => url,
                    None => {
                        return json!({
                            "error": "Config must include 'url' field",
                            "success": false
                        })
                    }
                };

                let default_data = json!({});
                let data = config_obj.get("data").unwrap_or(&default_data);

                let client = reqwest::blocking::Client::new();
                match client.post(url).json(data).send() {
                    Ok(response) => {
                        let status = response.status().as_u16();
                        let headers = response
                            .headers()
                            .iter()
                            .map(|(name, value)| {
                                (name.to_string(), value.to_str().unwrap_or("").to_string())
                            })
                            .collect::<std::collections::HashMap<String, String>>();

                        match response.text() {
                            Ok(body) => json!({
                                "url": url,
                                "status": status,
                                "body": body,
                                "headers": headers,
                                "success": true
                            }),
                            Err(e) => json!({
                                "url": url,
                                "error": format!("Failed to read response: {}", e),
                                "success": false
                            }),
                        }
                    }
                    Err(e) => json!({
                        "url": url,
                        "error": format!("Request failed: {}", e),
                        "success": false
                    }),
                }
            })
            .collect();

        Ok(json!(results))
    }

    #[cfg(not(feature = "parallel"))]
    {
        // Fallback to sequential requests
        let mut results = Vec::new();
        for config in configs {
            let config_obj = match config.as_object() {
                Some(obj) => obj,
                None => {
                    results.push(json!({
                        "error": "Each config must be an object with 'url' and 'data' fields",
                        "success": false
                    }));
                    continue;
                }
            };

            let url = match config_obj.get("url").and_then(|v| v.as_str()) {
                Some(url) => url,
                None => {
                    results.push(json!({
                        "error": "Config must include 'url' field",
                        "success": false
                    }));
                    continue;
                }
            };

            let default_data = json!({});
            let data = config_obj.get("data").unwrap_or(&default_data);

            let client = reqwest::blocking::Client::new();
            let result = match client.post(url).json(data).send() {
                Ok(response) => {
                    let status = response.status().as_u16();
                    let headers = response
                        .headers()
                        .iter()
                        .map(|(name, value)| {
                            (name.to_string(), value.to_str().unwrap_or("").to_string())
                        })
                        .collect::<std::collections::HashMap<String, String>>();

                    match response.text() {
                        Ok(body) => json!({
                            "url": url,
                            "status": status,
                            "body": body,
                            "headers": headers,
                            "success": true
                        }),
                        Err(e) => json!({
                            "url": url,
                            "error": format!("Failed to read response: {}", e),
                            "success": false
                        }),
                    }
                }
                Err(e) => json!({
                    "url": url,
                    "error": format!("Request failed: {}", e),
                    "success": false
                }),
            };
            results.push(result);
        }
        Ok(json!(results))
    }
}

// =============================================================================
// PARALLEL OPERATIONS (Auto-available for performance)
// =============================================================================

/// Read multiple files in parallel
fn builtin_core_read_files_parallel(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "readFilesParallel() expects exactly 1 argument (array of file paths)",
        ));
    }

    let paths = args[0].as_array().ok_or_else(|| {
        SusumuError::runtime_error("readFilesParallel() expects an array of file paths")
    })?;

    let path_strings: Result<Vec<&str>, _> = paths
        .iter()
        .map(|v| {
            v.as_str()
                .ok_or_else(|| SusumuError::runtime_error("All paths must be strings"))
        })
        .collect();
    let path_strings = path_strings?;

    #[cfg(feature = "parallel")]
    {
        use rayon::prelude::*;

        let results: Vec<Value> = path_strings
            .par_iter()
            .map(|&path| match std::fs::read_to_string(path) {
                Ok(content) => json!({
                    "path": path,
                    "content": content,
                    "success": true,
                    "size": content.len()
                }),
                Err(e) => json!({
                    "path": path,
                    "error": format!("Failed to read file: {}", e),
                    "success": false
                }),
            })
            .collect();

        Ok(json!(results))
    }

    #[cfg(not(feature = "parallel"))]
    {
        let mut results = Vec::new();
        for &path in &path_strings {
            let result = match std::fs::read_to_string(path) {
                Ok(content) => json!({
                    "path": path,
                    "content": content,
                    "success": true,
                    "size": content.len()
                }),
                Err(e) => json!({
                    "path": path,
                    "error": format!("Failed to read file: {}", e),
                    "success": false
                }),
            };
            results.push(result);
        }
        Ok(json!(results))
    }
}

/// Map array elements in parallel
fn builtin_core_map_parallel(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::runtime_error(
            "mapParallel() expects exactly 2 arguments (array, operation)",
        ));
    }

    let array = args[0].as_array().ok_or_else(|| {
        SusumuError::runtime_error("mapParallel() expects an array as first argument")
    })?;

    let operation = args[1].as_str().ok_or_else(|| {
        SusumuError::runtime_error("mapParallel() expects operation string as second argument")
    })?;

    #[cfg(feature = "parallel")]
    {
        use rayon::prelude::*;

        let results: Vec<Value> = array
            .par_iter()
            .map(|item| match operation {
                "double" => {
                    if let Some(n) = item.as_f64() {
                        json!(n * 2.0)
                    } else {
                        item.clone()
                    }
                }
                "square" => {
                    if let Some(n) = item.as_f64() {
                        json!(n * n)
                    } else {
                        item.clone()
                    }
                }
                _ => item.clone(),
            })
            .collect();

        Ok(json!(results))
    }

    #[cfg(not(feature = "parallel"))]
    {
        let results: Vec<Value> = array
            .iter()
            .map(|item| match operation {
                "double" => {
                    if let Some(n) = item.as_f64() {
                        json!(n * 2.0)
                    } else {
                        item.clone()
                    }
                }
                "square" => {
                    if let Some(n) = item.as_f64() {
                        json!(n * n)
                    } else {
                        item.clone()
                    }
                }
                _ => item.clone(),
            })
            .collect();

        Ok(json!(results))
    }
}

// =============================================================================
// STDLIB SUPPORT FUNCTIONS (For module compatibility)
// =============================================================================

/// Convert degrees to radians
fn builtin_to_radians(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "toRadians() expects exactly 1 argument",
        ));
    }

    let degrees = args[0]
        .as_f64()
        .ok_or_else(|| SusumuError::runtime_error("toRadians() expects a number"))?;

    Ok(json!(degrees * std::f64::consts::PI / 180.0))
}

/// Apply sine function
fn builtin_apply_sin(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "applySin() expects exactly 1 argument",
        ));
    }

    let radians = args[0]
        .as_f64()
        .ok_or_else(|| SusumuError::runtime_error("applySin() expects a number"))?;

    Ok(json!(radians.sin()))
}

/// Apply cosine function
fn builtin_apply_cos(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "applyCos() expects exactly 1 argument",
        ));
    }

    let radians = args[0]
        .as_f64()
        .ok_or_else(|| SusumuError::runtime_error("applyCos() expects a number"))?;

    Ok(json!(radians.cos()))
}

/// Apply tangent function
fn builtin_apply_tan(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 1 {
        return Err(SusumuError::runtime_error(
            "applyTan() expects exactly 1 argument",
        ));
    }

    let radians = args[0]
        .as_f64()
        .ok_or_else(|| SusumuError::runtime_error("applyTan() expects a number"))?;

    Ok(json!(radians.tan()))
}

/// Less than or equal comparison
fn builtin_less_than_or_equal(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::runtime_error(
            "lessThanOrEqual() expects exactly 2 arguments",
        ));
    }

    let a = args[0]
        .as_f64()
        .ok_or_else(|| SusumuError::runtime_error("lessThanOrEqual() expects numbers"))?;
    let b = args[1]
        .as_f64()
        .ok_or_else(|| SusumuError::runtime_error("lessThanOrEqual() expects numbers"))?;

    Ok(json!(a <= b))
}

/// Split string into parts (used by string module)
fn builtin_perform_split(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::runtime_error(
            "performSplit() expects exactly 2 arguments",
        ));
    }

    let text = args[0]
        .as_str()
        .ok_or_else(|| SusumuError::runtime_error("performSplit() expects a string"))?;
    let delimiter = args[1]
        .as_str()
        .ok_or_else(|| SusumuError::runtime_error("performSplit() expects delimiter string"))?;

    let parts: Vec<Value> = text.split(delimiter).map(|s| json!(s)).collect();

    Ok(json!(parts))
}

/// Split array into chunks (used by array module)
fn builtin_split_into_chunks(args: &[Value]) -> SusumuResult<Value> {
    if args.len() != 2 {
        return Err(SusumuError::runtime_error(
            "splitIntoChunks() expects exactly 2 arguments",
        ));
    }

    let array = args[0]
        .as_array()
        .ok_or_else(|| SusumuError::runtime_error("splitIntoChunks() expects an array"))?;
    let chunk_size = args[1]
        .as_f64()
        .ok_or_else(|| SusumuError::runtime_error("splitIntoChunks() expects chunk size"))?
        as usize;

    if chunk_size == 0 {
        return Err(SusumuError::runtime_error(
            "Chunk size must be greater than 0",
        ));
    }

    let chunks: Vec<Value> = array.chunks(chunk_size).map(|chunk| json!(chunk)).collect();

    Ok(json!(chunks))
}
