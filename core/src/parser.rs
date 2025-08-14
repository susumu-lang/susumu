//! Parser for Susumu arrow-flow language with type checking and visual debugging

use crate::ast::*;
use crate::error::{SusumuError, SusumuResult};
use crate::lexer::{Token, TokenType};
use crate::types::{SusumuType, TypeChecker, TypeError, TypeErrorKind};
// use std::collections::HashMap;

/// Enhanced parser with type checking and visual debugging
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    type_checker: TypeChecker,
    /// For visual debugging: track arrow flow paths
    arrow_flow_paths: Vec<ArrowFlowPath>,
}

/// Visual debugging information for arrow flows
#[derive(Debug, Clone)]
pub struct ArrowFlowPath {
    pub start_line: usize,
    pub start_column: usize,
    pub steps: Vec<ArrowFlowStep>,
    pub expected_types: Vec<SusumuType>,
    pub actual_types: Vec<SusumuType>,
}

#[derive(Debug, Clone)]
pub struct ArrowFlowStep {
    pub expression: String,
    pub direction: ArrowDirection,
    pub input_type: SusumuType,
    pub output_type: SusumuType,
    pub line: usize,
    pub column: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            type_checker: TypeChecker::new(),
            arrow_flow_paths: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> SusumuResult<Program> {
        let mut program = Program::new();
        
        // Skip initial newlines and comments
        self.skip_newlines_and_comments();

        while !self.is_at_end() {
            if self.match_token(&TokenType::Function) {
                let func = self.function_definition()?;
                // println!("DEBUG: Parsed function: {}", func.name);
                program.add_function(func);
            } else {
                // Check if this might be a function definition (identifier followed by parentheses and then brace)
                if self.check(&TokenType::Identifier) && 
                   self.tokens.get(self.current + 1).map_or(false, |t| t.token_type == TokenType::LeftParen) {
                    
                    // Look ahead to see if this is a function definition (has braces) or function call
                    let mut lookahead = self.current + 2; // Skip identifier and opening paren
                    let mut paren_count = 1;
                    
                    // Skip to closing paren
                    while lookahead < self.tokens.len() && paren_count > 0 {
                        match self.tokens[lookahead].token_type {
                            TokenType::LeftParen => paren_count += 1,
                            TokenType::RightParen => paren_count -= 1,
                            _ => {}
                        }
                        lookahead += 1;
                    }
                    
                    // Check if there's a brace after the closing paren (function definition)
                    let is_function_def = lookahead < self.tokens.len() && 
                        self.tokens[lookahead].token_type == TokenType::LeftBrace;
                    
                    if is_function_def {
                        let func = self.function_definition()?;
                        // println!("DEBUG: Parsed function (no keyword): {}", func.name);
                        program.add_function(func);
                    } else {
                        // It's a function call or other expression - treat as main expression
                        let expr = self.expression()?;
                        program.set_main_expression(expr);
                        break;
                    }
                } else {
                    // Check if this is a variable assignment (identifier = value)
                    if self.check(&TokenType::Identifier) && 
                       self.tokens.get(self.current + 1).map_or(false, |t| t.token_type == TokenType::Assign) {
                        let assignment = self.assignment_statement()?;
                        // Don't break - allow multiple top-level statements
                        if program.main_expression.is_none() {
                            program.set_main_expression(assignment);
                        } else {
                            // Create or extend a block expression for multiple statements
                            match program.main_expression.as_mut() {
                                Some(Expression::Block(exprs)) => {
                                    exprs.push(assignment);
                                }
                                Some(existing) => {
                                    let existing_expr = existing.clone();
                                    *existing = Expression::Block(vec![existing_expr, assignment]);
                                }
                                None => {
                                    program.set_main_expression(assignment);
                                }
                            }
                        }
                    } else {
                        // Main expression
                        let expr = self.expression()?;
                        if program.main_expression.is_none() {
                            program.set_main_expression(expr);
                        } else {
                            // Create or extend a block expression for multiple statements
                            match program.main_expression.as_mut() {
                                Some(Expression::Block(exprs)) => {
                                    exprs.push(expr);
                                }
                                Some(existing) => {
                                    let existing_expr = existing.clone();
                                    *existing = Expression::Block(vec![existing_expr, expr]);
                                }
                                None => {
                                    program.set_main_expression(expr);
                                }
                            }
                        }
                        // Only break if this was the last expression (no more tokens)
                        if self.is_at_end() {
                            break;
                        }
                    }
                }
            }
            
            self.skip_newlines_and_comments();
        }

        // Perform final type checking
        self.validate_program_types(&program)?;

        Ok(program)
    }

    /// Get visual debugging information for arrow flows
    pub fn get_arrow_flow_paths(&self) -> &[ArrowFlowPath] {
        &self.arrow_flow_paths
    }

    /// Generate visual arrow flow diagram
    pub fn generate_flow_diagram(&self, path: &ArrowFlowPath) -> String {
        let mut diagram = String::new();
        
        diagram.push_str(&format!("Arrow Flow Diagram (line {}):\n", path.start_line));
        diagram.push_str(&"=".repeat(50));
        diagram.push('\n');
        
        // Show the actual flow
        for (i, step) in path.steps.iter().enumerate() {
            if i == 0 {
                diagram.push_str(&format!("  {}", step.expression));
            } else {
                let arrow = match step.direction {
                    ArrowDirection::Forward => "->",
                    ArrowDirection::Backward => "<-",
                };
                diagram.push_str(&format!(" {} {}", arrow, step.expression));
            }
        }
        diagram.push('\n');
        
        // Show type flow
        diagram.push_str("Type Flow:\n");
        for (i, step) in path.steps.iter().enumerate() {
            let indent = "  ".repeat(i);
            diagram.push_str(&format!("{}Step {}: {} -> {}\n", 
                indent, i + 1, step.input_type.description(), step.output_type.description()));
        }
        
        // Show any type mismatches
        if !path.expected_types.is_empty() && !path.actual_types.is_empty() {
            diagram.push_str("\nType Analysis:\n");
            for (i, (expected, actual)) in path.expected_types.iter().zip(path.actual_types.iter()).enumerate() {
                let status = if expected.is_assignable_to(actual) { "✓" } else { "✗" };
                diagram.push_str(&format!("  {} Step {}: expected {}, got {}\n", 
                    status, i + 1, expected.description(), actual.description()));
            }
        }
        
        diagram
    }

    fn statement_or_expression(&mut self) -> SusumuResult<Expression> {
        // Check if this is an assignment statement
        if self.check(&TokenType::Identifier) {
            // Look ahead to see if next token is '='
            if self.tokens.get(self.current + 1).map_or(false, |t| t.token_type == TokenType::Assign) {
                return self.assignment_statement();
            }
        }
        // Otherwise, parse as expression
        self.expression()
    }

    fn assignment_statement(&mut self) -> SusumuResult<Expression> {
        let target_name = self.consume(&TokenType::Identifier, "Expected variable name")?.lexeme.clone();
        self.consume(&TokenType::Assign, "Expected '=' after variable name")?;
        
        // Skip any newlines before parsing the value
        self.skip_newlines_and_comments();
        
        let value = self.expression()?;
        
        Ok(Expression::Assignment {
            target: target_name,
            value: Box::new(value),
            mutable: false, // Default to immutable, could be extended with 'mut' keyword
        })
    }

    fn function_definition(&mut self) -> SusumuResult<FunctionDef> {
        let name = if self.check(&TokenType::Identifier) {
            self.advance().lexeme.clone()
        } else {
            self.consume(&TokenType::Identifier, "Expected function name")?.lexeme.clone()
        };
        
        self.consume(&TokenType::LeftParen, "Expected '(' after function name")?;
        
        let mut params = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                let param = self.consume(&TokenType::Identifier, "Expected parameter name")?.lexeme.clone();
                params.push(param);
                
                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }
        
        self.consume(&TokenType::RightParen, "Expected ')' after parameters")?;
        self.consume(&TokenType::LeftBrace, "Expected '{' before function body")?;
        
        self.skip_newlines_and_comments();
        
        // Parse multiple statements/expressions in function body
        let mut expressions = Vec::new();
        
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            expressions.push(self.statement_or_expression()?);
            self.skip_newlines_and_comments();
        }
        
        let body = if expressions.len() == 1 {
            expressions.into_iter().next().unwrap()
        } else {
            Expression::Block(expressions)
        };
        
        self.consume(&TokenType::RightBrace, "Expected '}' after function body")?;
        
        Ok(FunctionDef {
            name,
            params,
            body,
        })
    }

    fn expression(&mut self) -> SusumuResult<Expression> {
        self.annotation()
    }
    
    fn annotation(&mut self) -> SusumuResult<Expression> {
        if self.match_token(&TokenType::At) {
            let annotation = self.parse_annotation()?;
            self.skip_newlines_and_comments(); // Allow newlines after annotation
            let expression = self.conditional()?;
            Ok(Expression::Annotated {
                annotation,
                expression: Box::new(expression),
            })
        } else {
            self.conditional()
        }
    }
    
    fn parse_annotation(&mut self) -> SusumuResult<Annotation> {
        let annotation_name = self.consume(&TokenType::Identifier, "Expected annotation name after '@'")?;
        let annotation_type = annotation_name.lexeme.clone();
        let line = annotation_name.line;
        
        match annotation_type.as_str() {
            "trace" => {
                if self.match_token(&TokenType::LeftArrow) {
                    let value = self.primary()?;
                    if let Expression::String(trace_name) = value {
                        Ok(Annotation::Trace(trace_name))
                    } else {
                        Err(SusumuError::parser_error(
                            line,
                            "Expected string value for @trace annotation",
                        ))
                    }
                } else {
                    Err(SusumuError::parser_error(
                        line,
                        "Expected '<-' after @trace",
                    ))
                }
            }
            "monitor" => {
                if self.match_token(&TokenType::LeftArrow) {
                    let value = self.primary()?;
                    if let Expression::Array(items) = value {
                        let mut monitor_items = Vec::new();
                        for item in items {
                            if let Expression::String(s) = item {
                                monitor_items.push(s);
                            } else {
                                return Err(SusumuError::parser_error(
                                    line,
                                    "Expected array of strings for @monitor annotation",
                                ));
                            }
                        }
                        Ok(Annotation::Monitor(monitor_items))
                    } else {
                        Err(SusumuError::parser_error(
                            line,
                            "Expected array value for @monitor annotation",
                        ))
                    }
                } else {
                    Err(SusumuError::parser_error(
                        line,
                        "Expected '<-' after @monitor",
                    ))
                }
            }
            "config" => {
                if self.match_token(&TokenType::LeftArrow) {
                    let value = self.primary()?;
                    if let Expression::Object(pairs) = value {
                        // Convert to serde_json::Value
                        let mut map = serde_json::Map::new();
                        for (key, expr) in pairs {
                            let json_value = self.expression_to_json_value(expr)?;
                            map.insert(key, json_value);
                        }
                        Ok(Annotation::Config(serde_json::Value::Object(map)))
                    } else {
                        Err(SusumuError::parser_error(
                            line,
                            "Expected object value for @config annotation",
                        ))
                    }
                } else {
                    Err(SusumuError::parser_error(
                        line,
                        "Expected '<-' after @config",
                    ))
                }
            }
            "parallel" => {
                Ok(Annotation::Parallel)
            }
            "debug" => {
                if self.match_token(&TokenType::LeftArrow) {
                    let value = self.primary()?;
                    if let Expression::String(debug_label) = value {
                        Ok(Annotation::Debug(Some(debug_label)))
                    } else {
                        Err(SusumuError::parser_error(
                            line,
                            "Expected string value for @debug annotation",
                        ))
                    }
                } else {
                    Ok(Annotation::Debug(None))
                }
            }
            _ => Err(SusumuError::parser_error(
                line,
                format!("Unknown annotation type: @{}", annotation_type),
            ))
        }
    }
    
    fn expression_to_json_value(&self, expr: Expression) -> SusumuResult<serde_json::Value> {
        match expr {
            Expression::String(s) => Ok(serde_json::Value::String(s)),
            Expression::Number(n) => Ok(serde_json::json!(n)),
            Expression::Boolean(b) => Ok(serde_json::Value::Bool(b)),
            Expression::Null => Ok(serde_json::Value::Null),
            Expression::Array(items) => {
                let mut json_items = Vec::new();
                for item in items {
                    json_items.push(self.expression_to_json_value(item)?);
                }
                Ok(serde_json::Value::Array(json_items))
            }
            Expression::Object(pairs) => {
                let mut map = serde_json::Map::new();
                for (key, value) in pairs {
                    map.insert(key, self.expression_to_json_value(value)?);
                }
                Ok(serde_json::Value::Object(map))
            }
            _ => Err(SusumuError::runtime_error("Cannot convert expression to JSON value"))
        }
    }

    fn conditional(&mut self) -> SusumuResult<Expression> {
        let mut expr = self.arrow_chain()?;

        if self.match_token(&TokenType::I) {
            let condition_type = if self.check(&TokenType::Identifier) {
                let condition_name = self.advance().lexeme.clone();
                if condition_name == "success" {
                    ConditionType::Success
                } else {
                    ConditionType::Custom(condition_name)
                }
            } else {
                return Err(SusumuError::parser_error(
                    self.peek().line,
                    "Expected condition name after 'i'"
                ));
            };

            self.consume(&TokenType::LeftBrace, "Expected '{' after condition")?;
            self.skip_newlines_and_comments();
            
            let then_branch = self.parse_block_content()?;
            
            self.skip_newlines_and_comments();
            self.consume(&TokenType::RightBrace, "Expected '}' after then branch")?;
            
            // Parse else-if branches
            let mut else_if_branches = Vec::new();
            while self.match_token(&TokenType::Ei) {
                let else_if_condition_type = if self.check(&TokenType::Identifier) {
                    let condition_name = self.advance().lexeme.clone();
                    if condition_name == "success" {
                        ConditionType::Success
                    } else {
                        ConditionType::Custom(condition_name)
                    }
                } else {
                    return Err(SusumuError::parser_error(
                        self.peek().line,
                        "Expected condition name after 'ei'"
                    ));
                };

                self.consume(&TokenType::LeftBrace, "Expected '{' after else-if condition")?;
                self.skip_newlines_and_comments();
                
                let else_if_then_branch = self.parse_block_content()?;
                
                self.skip_newlines_and_comments();
                self.consume(&TokenType::RightBrace, "Expected '}' after else-if branch")?;
                
                else_if_branches.push(ElseIfBranch {
                    condition_type: else_if_condition_type,
                    condition: expr.clone(), // Use the same base expression
                    then_branch: else_if_then_branch,
                });
            }
            
            // Parse final else branch
            let else_branch = if self.match_token(&TokenType::E) {
                self.consume(&TokenType::LeftBrace, "Expected '{' after 'e'")?;
                self.skip_newlines_and_comments();
                
                let else_expr = self.parse_block_content()?;
                
                self.skip_newlines_and_comments();
                self.consume(&TokenType::RightBrace, "Expected '}' after else branch")?;
                
                Some(Box::new(else_expr))
            } else {
                None
            };

            // Type check conditional
            self.type_check_conditional(&expr, &condition_type, &then_branch, &else_branch)?;

            expr = Expression::Conditional {
                condition_type,
                condition: Box::new(expr),
                then_branch: Box::new(then_branch),
                else_if_branches,
                else_branch,
            };
        }

        Ok(expr)
    }

    fn arrow_chain(&mut self) -> SusumuResult<Expression> {
        let start_token = self.peek();
        let start_line = start_token.line;
        let start_column = start_token.column;
        
        let mut expressions = vec![self.postfix()?];
        let mut directions = Vec::new();
        let mut flow_steps = Vec::new();
        
        // Track the current type for visual debugging
        let current_type = SusumuType::Unknown; // Will be inferred
        
        while self.match_token(&TokenType::RightArrow) || self.match_token(&TokenType::LeftArrow) {
            let direction = if self.previous().token_type == TokenType::RightArrow {
                ArrowDirection::Forward
            } else {
                ArrowDirection::Backward
            };
            
            // Skip newlines after arrows to support multi-line arrow chains
            self.skip_newlines_and_comments();
            
            let next_expr = self.postfix()?;
            
            // Create flow step for visual debugging
            let step = ArrowFlowStep {
                expression: self.expression_to_string(&next_expr),
                direction: direction.clone(),
                input_type: current_type.clone(),
                output_type: SusumuType::Unknown, // Will be inferred
                line: self.previous().line,
                column: self.previous().column,
            };
            flow_steps.push(step);
            
            directions.push(direction);
            expressions.push(next_expr);
        }

        if directions.is_empty() {
            Ok(expressions.into_iter().next().unwrap())
        } else {
            // Type check the arrow chain
            let (expected_types, actual_types) = self.type_check_arrow_chain(&expressions, &directions)?;
            
            // Create visual debugging path
            let flow_path = ArrowFlowPath {
                start_line,
                start_column,
                steps: flow_steps,
                expected_types,
                actual_types,
            };
            self.arrow_flow_paths.push(flow_path);
            
            Ok(Expression::ArrowChain {
                expressions,
                directions,
            })
        }
    }

    fn postfix(&mut self) -> SusumuResult<Expression> {
        let mut expr = self.binary_op()?;
        
        // Handle property access: obj.property
        while self.match_token(&TokenType::Dot) {
            let property = self.consume(&TokenType::Identifier, "Expected property name after '.'")?;
            expr = Expression::PropertyAccess {
                object: Box::new(expr),
                property: property.lexeme.clone(),
            };
        }
        
        
        Ok(expr)
    }
    

    fn binary_op(&mut self) -> SusumuResult<Expression> {
        let mut expr = self.unary()?;

        while let Some(op) = self.match_binary_operator() {
            let right = self.unary()?;
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> SusumuResult<Expression> {
        if self.match_token(&TokenType::Minus) {
            let right = self.unary()?;
            Ok(Expression::BinaryOp {
                left: Box::new(Expression::Number(0.0)), // -x becomes 0 - x
                operator: BinaryOperator::Subtract,
                right: Box::new(right),
            })
        } else if self.match_token(&TokenType::Plus) {
            // Unary plus: +x just returns x
            self.unary()
        } else {
            self.foreach()
        }
    }

    fn match_binary_operator(&mut self) -> Option<BinaryOperator> {
        if self.match_token(&TokenType::Plus) {
            Some(BinaryOperator::Add)
        } else if self.match_token(&TokenType::Minus) {
            Some(BinaryOperator::Subtract)
        } else if self.match_token(&TokenType::Multiply) {
            Some(BinaryOperator::Multiply)
        } else if self.match_token(&TokenType::Divide) {
            Some(BinaryOperator::Divide)
        } else if self.match_token(&TokenType::Equal) {
            Some(BinaryOperator::Equal)
        } else if self.match_token(&TokenType::NotEqual) {
            Some(BinaryOperator::NotEqual)
        } else if self.match_token(&TokenType::Less) {
            Some(BinaryOperator::Less)
        } else if self.match_token(&TokenType::Greater) {
            Some(BinaryOperator::Greater)
        } else if self.match_token(&TokenType::LessEq) {
            Some(BinaryOperator::LessEq)
        } else if self.match_token(&TokenType::GreaterEq) {
            Some(BinaryOperator::GreaterEq)
        } else {
            None
        }
    }

    fn foreach(&mut self) -> SusumuResult<Expression> {
        if self.match_token(&TokenType::Match) {
            self.match_expression()
        } else if self.match_token(&TokenType::ForEach) {
            let variable = self.consume(&TokenType::Identifier, "Expected variable name after 'fe'")?.lexeme.clone();
            self.consume(&TokenType::In, "Expected 'in' after foreach variable")?;
            
            let iterable = self.primary()?;
            
            self.consume(&TokenType::LeftBrace, "Expected '{' after iterable")?;
            self.skip_newlines_and_comments();
            
            let body = self.expression()?;
            
            self.skip_newlines_and_comments();
            self.consume(&TokenType::RightBrace, "Expected '}' after foreach body")?;
            
            Ok(Expression::ForEach {
                variable,
                iterable: Box::new(iterable),
                body: Box::new(body),
            })
        } else {
            self.flow_control()
        }
    }

    fn flow_control(&mut self) -> SusumuResult<Expression> {
        if self.match_token(&TokenType::Return) {
            self.consume(&TokenType::LeftArrow, "Expected '<-' after 'return'")?;
            let value = self.expression()?;  // Parse full expression, not just primary
            Ok(Expression::Return(Box::new(value)))
        } else if self.match_token(&TokenType::Error) {
            self.consume(&TokenType::LeftArrow, "Expected '<-' after 'error'")?;
            let value = self.expression()?;  // Parse full expression, not just primary
            Ok(Expression::Error(Box::new(value)))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> SusumuResult<Expression> {
        if self.match_token(&TokenType::Number) {
            let lexeme = &self.previous().lexeme;
            // Try to parse as integer first, fall back to float
            let value = if lexeme.contains('.') {
                lexeme.parse::<f64>()
                    .map_err(|_| SusumuError::parser_error(
                        self.previous().line,
                        "Invalid number format"
                    ))?
            } else {
                // Parse as integer, but store as f64 for compatibility
                lexeme.parse::<i64>()
                    .map_err(|_| SusumuError::parser_error(
                        self.previous().line,
                        "Invalid number format"
                    ))? as f64
            };
            Ok(Expression::Number(value))
        } else if self.match_token(&TokenType::String) {
            Ok(Expression::String(self.previous().lexeme.clone()))
        } else if self.match_token(&TokenType::True) {
            Ok(Expression::Boolean(true))
        } else if self.match_token(&TokenType::False) {
            Ok(Expression::Boolean(false))
        } else if self.match_token(&TokenType::Null) {
            Ok(Expression::Null)
        } else if self.match_token(&TokenType::Identifier) {
            let name = self.previous().lexeme.clone();
            
            if self.match_token(&TokenType::LeftParen) {
                // Function call
                let mut args = Vec::new();
                
                if !self.check(&TokenType::RightParen) {
                    loop {
                        args.push(self.expression()?);
                        if !self.match_token(&TokenType::Comma) {
                            break;
                        }
                    }
                }
                
                self.consume(&TokenType::RightParen, "Expected ')' after function arguments")?;
                
                Ok(Expression::FunctionCall { name, args })
            } else {
                Ok(Expression::Identifier(name))
            }
        } else if self.match_token(&TokenType::LeftParen) {
            // Tuple or grouped expression
            if self.check(&TokenType::RightParen) {
                // Empty tuple
                self.advance();
                Ok(Expression::Tuple(Vec::new()))
            } else {
                let first_expr = self.expression()?;
                
                if self.match_token(&TokenType::Comma) {
                    // Tuple
                    let mut elements = vec![first_expr];
                    
                    if !self.check(&TokenType::RightParen) {
                        loop {
                            elements.push(self.expression()?);
                            if !self.match_token(&TokenType::Comma) {
                                break;
                            }
                        }
                    }
                    
                    self.consume(&TokenType::RightParen, "Expected ')' after tuple elements")?;
                    Ok(Expression::Tuple(elements))
                } else {
                    // Grouped expression
                    self.consume(&TokenType::RightParen, "Expected ')' after expression")?;
                    Ok(first_expr)
                }
            }
        } else if self.match_token(&TokenType::LeftBrace) {
            // Determine if this is an object literal or block expression
            if self.is_object_literal() {
                // Object literal: {key: value, key2: value2}
                let mut pairs = Vec::new();
                
                // Skip newlines after opening brace
                self.skip_newlines_and_comments();
                
                if !self.check(&TokenType::RightBrace) {
                    loop {
                        let key = if self.check(&TokenType::Identifier) {
                            self.advance().lexeme.clone()
                        } else if self.check(&TokenType::String) {
                            self.advance().lexeme.clone()
                        } else {
                            return Err(SusumuError::parser_error(
                                self.peek().line,
                                "Expected property name"
                            ));
                        };
                        
                        self.consume(&TokenType::Colon, "Expected ':' after property name")?;
                        let value = self.expression()?;
                        
                        pairs.push((key, value));
                        
                        if !self.match_token(&TokenType::Comma) {
                            break;
                        }
                        
                        // Skip newlines after comma
                        self.skip_newlines_and_comments();
                    }
                }
                
                // Skip newlines before closing brace
                self.skip_newlines_and_comments();
                
                self.consume(&TokenType::RightBrace, "Expected '}' after object literal")?;
                Ok(Expression::Object(pairs))
            } else {
                // Block expression: { expr1; expr2; expr3 }
                self.parse_block_expression()
            }
        } else if self.match_token(&TokenType::LeftBracket) {
            // Array literal
            let mut elements = Vec::new();
            
            // Skip newlines after opening bracket
            self.skip_newlines_and_comments();
            
            if !self.check(&TokenType::RightBracket) {
                loop {
                    elements.push(self.expression()?);
                    if !self.match_token(&TokenType::Comma) {
                        break;
                    }
                    
                    // Skip newlines after comma
                    self.skip_newlines_and_comments();
                }
            }
            
            // Skip newlines before closing bracket
            self.skip_newlines_and_comments();
            
            self.consume(&TokenType::RightBracket, "Expected ']' after array elements")?;
            Ok(Expression::Array(elements))
        } else if self.match_token(&TokenType::I) {
            // Standalone conditional: i condition { ... } ei condition { ... } e { ... }
            // This handles cases like: i success { ... } when not preceded by an expression
            let condition_type = if self.check(&TokenType::Identifier) {
                let condition_name = self.advance().lexeme.clone();
                if condition_name == "success" {
                    ConditionType::Success
                } else {
                    ConditionType::Custom(condition_name)
                }
            } else {
                return Err(SusumuError::parser_error(
                    self.peek().line,
                    "Expected condition name after 'i'"
                ));
            };

            self.consume(&TokenType::LeftBrace, "Expected '{' after condition")?;
            self.skip_newlines_and_comments();
            
            let then_branch = self.parse_block_content()?;
            
            self.skip_newlines_and_comments();
            self.consume(&TokenType::RightBrace, "Expected '}' after then branch")?;
            
            // Parse else-if branches
            let mut else_if_branches = Vec::new();
            while self.match_token(&TokenType::Ei) {
                let else_if_condition_type = if self.check(&TokenType::Identifier) {
                    let condition_name = self.advance().lexeme.clone();
                    if condition_name == "success" {
                        ConditionType::Success
                    } else {
                        ConditionType::Custom(condition_name)
                    }
                } else {
                    return Err(SusumuError::parser_error(
                        self.peek().line,
                        "Expected condition name after 'ei'"
                    ));
                };

                self.consume(&TokenType::LeftBrace, "Expected '{' after else-if condition")?;
                self.skip_newlines_and_comments();
                
                let else_if_then_branch = self.parse_block_content()?;
                
                self.skip_newlines_and_comments();
                self.consume(&TokenType::RightBrace, "Expected '}' after else-if branch")?;
                
                // For standalone conditionals, we use a null condition placeholder
                else_if_branches.push(ElseIfBranch {
                    condition_type: else_if_condition_type,
                    condition: Expression::Null,
                    then_branch: else_if_then_branch,
                });
            }
            
            // Parse final else branch
            let else_branch = if self.match_token(&TokenType::E) {
                self.consume(&TokenType::LeftBrace, "Expected '{' after 'e'")?;
                self.skip_newlines_and_comments();
                
                let else_expr = self.parse_block_content()?;
                
                self.skip_newlines_and_comments();
                self.consume(&TokenType::RightBrace, "Expected '}' after else branch")?;
                
                Some(Box::new(else_expr))
            } else {
                None
            };

            // For standalone conditionals, we need a placeholder condition
            // The actual condition will be determined by context (usually a preceding arrow chain)
            Ok(Expression::Conditional {
                condition_type,
                condition: Box::new(Expression::Null), // Placeholder - will be replaced by arrow chain evaluation
                then_branch: Box::new(then_branch),
                else_if_branches,
                else_branch,
            })
        } else {
            Err(self.error_with_suggestion("Unexpected token"))
        }
    }

    // Type checking methods
    fn type_check_conditional(
        &mut self,
        condition: &Expression,
        condition_type: &ConditionType,
        _then_branch: &Expression,
        _else_branch: &Option<Box<Expression>>,
    ) -> SusumuResult<()> {
        match condition_type {
            ConditionType::Success => {
                // For 'i success', the condition should be a result type
                let condition_type = self.infer_expression_type(condition)?;
                if !condition_type.is_result() {
                    return Err(SusumuError::parser_error(
                        self.peek().line,
                        &format!("Expected result type for 'i success', found: {}", condition_type.description())
                    ));
                }
            }
            ConditionType::Custom(_name) => {
                // Custom conditions - validate against known condition functions
                // This could be extended with a registry of valid condition functions
            }
            ConditionType::If => {
                // Traditional if - condition should be boolean-like
            }
        }
        
        Ok(())
    }

    fn type_check_arrow_chain(
        &mut self,
        expressions: &[Expression],
        directions: &[ArrowDirection],
    ) -> SusumuResult<(Vec<SusumuType>, Vec<SusumuType>)> {
        let mut expected_types = Vec::new();
        let mut actual_types = Vec::new();
        
        // Infer types for all expressions
        for expr in expressions {
            let expr_type = self.infer_expression_type(expr)?;
            actual_types.push(expr_type);
        }
        
        // Check arrow chain compatibility
        let mut current_type = actual_types[0].clone();
        expected_types.push(current_type.clone());
        
        for (i, direction) in directions.iter().enumerate() {
            let target_expr = &expressions[i + 1];
            let target_type = &actual_types[i + 1];
            
            match direction {
                ArrowDirection::Forward => {
                    // For forward arrows: current -> function
                    // The function should accept the current type
                    if let Expression::Identifier(func_name) = target_expr {
                        let expected_input = self.get_function_input_type(func_name)?;
                        expected_types.push(expected_input.clone());
                        
                        if !current_type.is_assignable_to(&expected_input) {
                            self.add_type_error(TypeError {
                                line: self.peek().line,
                                column: self.peek().column,
                                error_type: TypeErrorKind::ArrowChainError {
                                    step: i + 1,
                                    expected_input: expected_input.clone(),
                                    actual_input: current_type.clone(),
                                    function_name: func_name.clone(),
                                },
                                suggestion: format!(
                                    "Convert {} to {} or use a different function",
                                    current_type.description(),
                                    expected_input.description()
                                ),
                            });
                        }
                        
                        current_type = self.get_function_output_type(func_name)?;
                    }
                }
                ArrowDirection::Backward => {
                    // For backward arrows: function <- argument
                    // Handled in convergence analysis
                    expected_types.push(target_type.clone());
                }
            }
        }
        
        Ok((expected_types, actual_types))
    }

    fn infer_expression_type(&self, expr: &Expression) -> SusumuResult<SusumuType> {
        match expr {
            Expression::Number(_) => Ok(SusumuType::Number),
            Expression::String(_) => Ok(SusumuType::String),
            Expression::Boolean(_) => Ok(SusumuType::Boolean),
            Expression::Null => Ok(SusumuType::Null),
            Expression::Identifier(name) => {
                // Look up variable or function type
                if let Some(var_type) = self.type_checker.env.get_variable(name) {
                    Ok(var_type.clone())
                } else if let Some(func_type) = self.type_checker.env.get_function(name) {
                    Ok(func_type.clone())
                } else {
                    Ok(SusumuType::Unknown)
                }
            }
            Expression::Tuple(elements) => {
                let element_types: Result<Vec<_>, _> = elements.iter()
                    .map(|e| self.infer_expression_type(e))
                    .collect();
                Ok(SusumuType::Tuple(element_types?))
            }
            Expression::Array(elements) => {
                if elements.is_empty() {
                    Ok(SusumuType::Array(Box::new(SusumuType::Unknown)))
                } else {
                    let first_type = self.infer_expression_type(&elements[0])?;
                    Ok(SusumuType::Array(Box::new(first_type)))
                }
            }
            Expression::Object(pairs) => {
                let mut field_types = Vec::new();
                for (key, value_expr) in pairs {
                    let value_type = self.infer_expression_type(value_expr)?;
                    field_types.push((key.clone(), value_type));
                }
                Ok(SusumuType::Object(field_types))
            }
            Expression::FunctionCall { name, args: _ } => {
                self.get_function_output_type(name)
            }
            _ => Ok(SusumuType::Unknown),
        }
    }

    fn get_function_input_type(&self, name: &str) -> SusumuResult<SusumuType> {
        // Simplified - in reality, would lookup from type environment
        match name {
            "add" | "multiply" | "subtract" | "divide" => Ok(SusumuType::Number),
            "print" => Ok(SusumuType::Unknown), // Accepts anything
            "length" => Ok(SusumuType::Union(vec![
                SusumuType::String,
                SusumuType::Array(Box::new(SusumuType::Unknown))
            ])),
            _ => Ok(SusumuType::Unknown),
        }
    }

    fn get_function_output_type(&self, name: &str) -> SusumuResult<SusumuType> {
        match name {
            "add" | "multiply" | "subtract" | "divide" => Ok(SusumuType::Number),
            "print" => Ok(SusumuType::Unknown),
            "length" => Ok(SusumuType::Number),
            "first" => Ok(SusumuType::Union(vec![SusumuType::Unknown, SusumuType::Null])),
            _ => Ok(SusumuType::Unknown),
        }
    }

    fn validate_program_types(&self, _program: &Program) -> SusumuResult<()> {
        // Perform global type validation
        // This would include checking function definitions, main expression, etc.
        Ok(())
    }

    /// Look ahead to determine if this is an object literal or block expression
    fn is_object_literal(&self) -> bool {
        if self.check(&TokenType::RightBrace) {
            return true; // Empty object {}
        }
        
        // Look for pattern: identifier/string followed by colon, skipping newlines
        let mut pos = self.current;
        
        // Skip newlines to find the first non-newline token
        while pos < self.tokens.len() && self.tokens[pos].token_type == TokenType::Newline {
            pos += 1;
        }
        
        if pos < self.tokens.len() {
            let first_token = &self.tokens[pos];
            if matches!(first_token.token_type, TokenType::Identifier | TokenType::String) {
                // Look for colon after first token, again skipping newlines
                pos += 1;
                while pos < self.tokens.len() && self.tokens[pos].token_type == TokenType::Newline {
                    pos += 1;
                }
                if pos < self.tokens.len() {
                    return self.tokens[pos].token_type == TokenType::Colon;
                }
            }
        }
        
        false // Default to block expression
    }

    /// Parse a block expression: { expr1; expr2; expr3 }
    fn parse_block_expression(&mut self) -> SusumuResult<Expression> {
        let mut expressions = Vec::new();
        
        self.skip_newlines_and_comments();
        
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            expressions.push(self.expression()?);
            self.skip_newlines_and_comments();
        }
        
        self.consume(&TokenType::RightBrace, "Expected '}' after block expression")?;
        
        if expressions.len() == 1 {
            Ok(expressions.into_iter().next().unwrap())
        } else {
            Ok(Expression::Block(expressions))
        }
    }

    fn parse_block_content(&mut self) -> SusumuResult<Expression> {
        let mut expressions = Vec::new();
        
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            expressions.push(self.expression()?);
            self.skip_newlines_and_comments();
        }
        
        if expressions.len() == 1 {
            Ok(expressions.into_iter().next().unwrap())
        } else {
            Ok(Expression::Block(expressions))
        }
    }

    fn add_type_error(&mut self, error: TypeError) {
        // In a real implementation, would collect and report type errors
        eprintln!("{}", self.type_checker.generate_error_message(&error));
    }

    fn match_expression(&mut self) -> SusumuResult<Expression> {
        // Check if there's an expression to match on
        let expr = if self.check(&TokenType::LeftBrace) {
            // No expression, match on previous arrow chain result
            None
        } else {
            // Parse the expression to match on
            Some(Box::new(self.primary()?))
        };

        self.consume(&TokenType::LeftBrace, "Expected '{' after 'match'")?;
        self.skip_newlines_and_comments();

        let mut cases = Vec::new();
        
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            let case = self.match_case()?;
            cases.push(case);
            self.skip_newlines_and_comments();
        }

        self.consume(&TokenType::RightBrace, "Expected '}' after match cases")?;

        Ok(Expression::Match { expr, cases })
    }

    fn match_case(&mut self) -> SusumuResult<MatchCase> {
        let pattern = self.pattern()?;
        
        // Optional guard condition
        let guard = if self.match_token(&TokenType::When) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(&TokenType::RightArrow, "Expected '->' after pattern")?;
        let body = self.expression()?;

        Ok(MatchCase {
            pattern,
            guard,
            body,
        })
    }

    fn pattern(&mut self) -> SusumuResult<Pattern> {
        if self.match_token(&TokenType::Underscore) {
            Ok(Pattern::Wildcard)
        } else if self.match_token(&TokenType::Identifier) || 
                  self.match_token(&TokenType::Error) ||
                  self.match_token(&TokenType::Return) {
            let name = self.previous().lexeme.clone();
            if name == "some" || name == "none" || name == "success" || name == "error" {
                // Check for arrow pattern syntax: some <- x  or parentheses syntax: some(x)
                if self.check(&TokenType::LeftArrow) {
                    // Arrow pattern syntax: some <- x
                    self.advance(); // consume <-
                    if name == "none" {
                        // none doesn't take an argument
                        Ok(Pattern::ArrowPattern {
                            constructor: name,
                            arg: Box::new(Pattern::Wildcard),
                        })
                    } else {
                        let inner_pattern = self.pattern()?;
                        Ok(Pattern::ArrowPattern {
                            constructor: name,
                            arg: Box::new(inner_pattern),
                        })
                    }
                } else if self.check(&TokenType::LeftParen) {
                    // Parentheses syntax: some(x)
                    self.advance(); // consume (
                    let inner_pattern = if name == "none" {
                        Pattern::Wildcard
                    } else {
                        self.pattern()?
                    };
                    self.consume(&TokenType::RightParen, &format!("Expected ')' after {} pattern", name))?;
                    Ok(Pattern::ArrowPattern {
                        constructor: name,
                        arg: Box::new(inner_pattern),
                    })
                } else if name == "none" {
                    // Plain 'none' pattern
                    Ok(Pattern::ArrowPattern {
                        constructor: name,
                        arg: Box::new(Pattern::Wildcard),
                    })
                } else {
                    // Constructor name without arrow or parens is just an identifier
                    Ok(Pattern::Identifier(name))
                }
            } else {
                Ok(Pattern::Identifier(name))
            }
        } else if self.match_token(&TokenType::LeftParen) {
            // Tuple pattern
            let mut patterns = Vec::new();
            
            if !self.check(&TokenType::RightParen) {
                loop {
                    patterns.push(self.pattern()?);
                    if !self.match_token(&TokenType::Comma) {
                        break;
                    }
                }
            }
            
            self.consume(&TokenType::RightParen, "Expected ')' after tuple pattern")?;
            Ok(Pattern::Tuple(patterns))
        } else if self.match_token(&TokenType::LeftBrace) {
            // Object pattern
            let mut field_patterns = Vec::new();
            
            if !self.check(&TokenType::RightBrace) {
                loop {
                    let key = self.consume(&TokenType::Identifier, "Expected field name")?.lexeme.clone();
                    self.consume(&TokenType::Colon, "Expected ':' after field name")?;
                    let pattern = self.pattern()?;
                    field_patterns.push((key, pattern));
                    
                    if !self.match_token(&TokenType::Comma) {
                        break;
                    }
                }
            }
            
            self.consume(&TokenType::RightBrace, "Expected '}' after object pattern")?;
            Ok(Pattern::Object(field_patterns))
        } else {
            // Literal pattern
            let literal = if self.match_token(&TokenType::Number) {
                let value = self.previous().lexeme.parse::<f64>()
                    .map_err(|_| SusumuError::parser_error(
                        self.previous().line,
                        "Invalid number in pattern"
                    ))?;
                LiteralValue::Number(value)
            } else if self.match_token(&TokenType::String) {
                LiteralValue::String(self.previous().lexeme.clone())
            } else if self.match_token(&TokenType::True) {
                LiteralValue::Boolean(true)
            } else if self.match_token(&TokenType::False) {
                LiteralValue::Boolean(false)
            } else if self.match_token(&TokenType::Null) {
                LiteralValue::Null
            } else {
                return Err(SusumuError::parser_error(
                    self.peek().line,
                    "Expected pattern"
                ));
            };
            
            Ok(Pattern::Literal(literal))
        }
    }

    // Helper methods
    fn expression_to_string(&self, expr: &Expression) -> String {
        match expr {
            Expression::Identifier(name) => name.clone(),
            Expression::Number(n) => n.to_string(),
            Expression::String(s) => format!("\"{}\"", s),
            Expression::Boolean(b) => b.to_string(),
            Expression::Null => "null".to_string(),
            _ => "expression".to_string(),
        }
    }

    fn skip_newlines_and_comments(&mut self) {
        while self.match_token(&TokenType::Newline) || self.match_token(&TokenType::Comment) {
            // Skip
        }
    }

    fn match_token(&mut self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.peek().token_type == token_type
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> SusumuResult<&Token> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(self.error_with_suggestion(message))
        }
    }

    fn error_with_suggestion(&self, message: &str) -> SusumuError {
        let current_token = self.peek();
        let prev_token = if self.current > 0 { 
            &self.tokens[self.current - 1] 
        } else { 
            current_token 
        };
        
        let (detailed_message, suggestion) = self.generate_detailed_error(&current_token.token_type, message, prev_token);
        
        SusumuError::parser_error(
            current_token.line,
            &format!("{} at column {}\n💡 {}\n🔍 Context: Previous token was '{}' ({}:{})", 
                     detailed_message, 
                     current_token.column,
                     suggestion,
                     prev_token.lexeme,
                     prev_token.line,
                     prev_token.column)
        )
    }

    fn generate_detailed_error(&self, found_token: &TokenType, message: &str, prev_token: &Token) -> (String, String) {
        match found_token {
            TokenType::LeftBrace => {
                if prev_token.token_type == TokenType::RightArrow {
                    ("Block expression after '->' arrow".to_string(), 
                     "Blocks after arrows should contain expressions: `expr -> { stmt1; stmt2 }`".to_string())
                } else {
                    ("Unexpected '{' found".to_string(),
                     "Use '{key: value}' for objects or '{ expr1; expr2 }' for blocks".to_string())
                }
            },
            TokenType::RightBrace => ("Unexpected '}' - missing opening brace or unmatched braces".to_string(),
                                    "Check that all '{' have matching '}' and blocks contain valid expressions".to_string()),
            TokenType::Identifier => {
                let context = if prev_token.token_type == TokenType::LeftArrow {
                    "after '<-' arrow - this should be a variable binding in patterns"
                } else if prev_token.token_type == TokenType::RightArrow {
                    "after '->' arrow - this should be a function or expression"
                } else {
                    "identifier"
                };
                (format!("Unexpected identifier '{}' {}", self.peek().lexeme, context),
                 "Check spelling, ensure variables are defined, and verify function exists".to_string())
            },
            TokenType::RightArrow => ("Unexpected '->' arrow".to_string(),
                                    "Arrows flow data: `value -> function` or `value -> function <- args`".to_string()),
            TokenType::LeftArrow => ("Unexpected '<-' arrow".to_string(),
                                   "Backward arrows provide convergent input: `main -> func <- arg1 <- arg2`".to_string()),
            TokenType::Dot => ("Property access not yet supported".to_string(),
                              "Currently implementing: `obj.property` syntax - use workaround for now".to_string()),
            TokenType::EOF => ("Unexpected end of file".to_string(),
                              "Expression or statement appears incomplete - check for missing closing braces".to_string()),
            _ => (format!("{} - found '{}' of type {:?}", message, self.peek().lexeme, found_token),
                  "Check the Susumu syntax guide for correct patterns".to_string()),
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_basic_arrow_chain_parsing() {
        let source = "5 -> add <- 3";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        
        assert!(ast.main_expression.is_some());
        if let Some(Expression::ArrowChain { expressions, directions }) = ast.main_expression {
            assert_eq!(expressions.len(), 3);
            assert_eq!(directions.len(), 2);
            assert_eq!(directions[0], ArrowDirection::Forward);
            assert_eq!(directions[1], ArrowDirection::Backward);
        } else {
            panic!("Expected arrow chain");
        }
    }

    #[test]
    fn test_conditional_parsing() {
        let source = "x -> validate -> i success { result -> return } e { error -> error }";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        
        assert!(ast.main_expression.is_some());
    }

    #[test]
    fn test_visual_debugging_info() {
        let source = "5 -> add <- 3 -> multiply <- 2";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let _ast = parser.parse().unwrap();
        
        let flow_paths = parser.get_arrow_flow_paths();
        assert!(!flow_paths.is_empty());
        
        let diagram = parser.generate_flow_diagram(&flow_paths[0]);
        assert!(diagram.contains("Arrow Flow Diagram"));
        assert!(diagram.contains("Type Flow"));
    }

    #[test]
    fn test_type_error_detection() {
        let source = r#""hello" -> add <- 5"#; // String cannot be added to number
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        
        // Parser should complete but type checker should flag errors
        let _result = parser.parse();
        // In a real implementation, would check for type errors
    }
}