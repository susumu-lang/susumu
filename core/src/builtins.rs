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

        registry.register_math_functions();
        registry.register_string_functions();
        registry.register_array_functions();
        registry.register_io_functions();
        registry.register_utility_functions();
        registry.register_maybe_result_functions();

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
