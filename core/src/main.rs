//! Susumu Programming Language CLI

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;
use susumu::{execute_to_string, Interpreter, Lexer, Parser};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => run_repl(),
        2 => {
            if args[1] == "--debug" || args[1] == "-d" {
                eprintln!("Usage: {} [--debug] <script.susu>", args[0]);
                process::exit(1);
            }
            run_file(&args[1], false)
        }
        3 => {
            if args[1] == "--debug" || args[1] == "-d" {
                run_file(&args[2], true)
            } else {
                eprintln!("Usage: {} [--debug] <script.susu>", args[0]);
                process::exit(1);
            }
        }
        _ => {
            eprintln!("Usage: {} [--debug] <script.susu>", args[0]);
            process::exit(1);
        }
    }
}

fn run_file(filename: &str, debug_mode: bool) {
    match fs::read_to_string(filename) {
        Ok(source) => {
            if debug_mode {
                run_file_with_debug(&source);
            } else {
                let result = execute_to_string(&source);
                println!("{}", result);
            }
        }
        Err(err) => {
            eprintln!("Error reading file '{}': {}", filename, err);
            process::exit(1);
        }
    }
}

fn run_file_with_debug(source: &str) {
    match execute_with_debugging(source) {
        Ok((result, traces, stats)) => {
            // Show the result
            println!("🎯 Result: {:?}", result);

            // Show performance stats
            println!("\n📊 Performance Statistics:");
            println!(
                "   • Expressions evaluated: {}",
                stats.total_expressions_evaluated
            );
            println!(
                "   • Execution time: {}μs",
                stats.total_execution_time_ns / 1000
            );
            println!("   • Arrow chains: {}", stats.arrow_chain_count);
            println!("   • Function calls: {}", stats.function_call_count);
            println!(
                "   • Convergence operations: {}",
                stats.convergence_operations
            );

            // Show execution traces
            if !traces.is_empty() {
                println!("\n🔍 Execution Flow:");
                for (i, trace) in traces.iter().enumerate() {
                    println!(
                        "   {}. {} -> {}",
                        i + 1,
                        trace.expression,
                        value_to_display_string(&trace.output_value)
                    );
                }
            }

            // Generate flow diagram
            let mut interpreter = Interpreter::new();
            if let Ok(tokens) = Lexer::new(source).tokenize() {
                if let Ok(ast) = Parser::new(tokens).parse() {
                    let _ = interpreter.execute(&ast);
                    let diagram = interpreter.generate_execution_diagram();
                    println!("\n🏗️  {}", diagram);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
        }
    }
}

fn execute_with_debugging(
    source: &str,
) -> Result<
    (
        serde_json::Value,
        Vec<susumu::interpreter::ExecutionTrace>,
        susumu::interpreter::PerformanceStats,
    ),
    Box<dyn std::error::Error>,
> {
    let tokens = Lexer::new(source).tokenize()?;
    let ast = Parser::new(tokens).parse()?;
    let mut interpreter = Interpreter::new();

    let result = interpreter.execute(&ast)?;
    let traces = interpreter.get_execution_traces().to_vec();
    let stats = interpreter.get_performance_stats().clone();

    Ok((result, traces, stats))
}

fn value_to_display_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("\"{}\"", s),
        serde_json::Value::Array(_) => "[array]".to_string(),
        serde_json::Value::Object(_) => "{object}".to_string(),
    }
}

fn run_repl() {
    println!("Susumu Programming Language v0.1.0");
    println!("Arrow-flow programming with visual data transformations");
    println!("Type 'exit' or 'quit' to exit, 'help' for help");
    println!();

    let mut interpreter = Interpreter::new();
    let mut line_number = 1;

    loop {
        print!("susumu:{:03}> ", line_number);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();

                if input.is_empty() {
                    continue;
                }

                match input {
                    "exit" | "quit" => {
                        println!("Goodbye!");
                        break;
                    }
                    "help" => {
                        print_help();
                        continue;
                    }
                    "clear" => {
                        print!("\x1B[2J\x1B[1;1H");
                        continue;
                    }
                    _ => {}
                }

                match execute_repl_line(input, &mut interpreter) {
                    Ok(result) => {
                        if result != "null" {
                            println!("=> {}", result);
                        }
                    }
                    Err(err) => {
                        eprintln!("Error: {}", err);
                    }
                }

                line_number += 1;
            }
            Err(err) => {
                eprintln!("Error reading input: {}", err);
                break;
            }
        }
    }
}

fn execute_repl_line(source: &str, interpreter: &mut Interpreter) -> Result<String, String> {
    // Try to parse as an expression first
    match Lexer::new(source).tokenize() {
        Ok(tokens) => match Parser::new(tokens).parse() {
            Ok(ast) => match interpreter.execute(&ast) {
                Ok(result) => {
                    Ok(serde_json::to_string(&result).unwrap_or_else(|_| "null".to_string()))
                }
                Err(err) => Err(err.to_string()),
            },
            Err(err) => Err(err.to_string()),
        },
        Err(err) => Err(err.to_string()),
    }
}

fn print_help() {
    println!("Susumu Programming Language Help");
    println!("================================");
    println!();
    println!("Basic Syntax:");
    println!("  Arrow chains:     5 -> add <- 3 -> multiply <- 2");
    println!("  Function calls:   multiply(5, 3)");
    println!("  Conditionals:     i success {{ ... }} e {{ ... }}");
    println!("  Functions:        myFunc(x) {{ x -> add <- 1 -> return }}");
    println!();
    println!("Built-in Functions:");
    println!("  Math:             add, subtract, multiply, divide, power, sqrt");
    println!("  Strings:          concat, length, substring, to_upper, to_lower");
    println!("  Arrays:           first, last, push, pop, sort, reverse");
    println!("  I/O:              print, println, debug");
    println!("  Utility:          type_of, is_null, is_number, to_string");
    println!();
    println!("Examples:");
    println!("  5 -> add <- 3                    # Basic arrow chain");
    println!("  [1,2,3] -> first -> print        # Array operations");
    println!("  \"hello\" -> to_upper -> print      # String operations");
    println!();
    println!("REPL Commands:");
    println!("  help              Show this help message");
    println!("  clear             Clear the screen");
    println!("  exit, quit        Exit the REPL");
    println!();
}
