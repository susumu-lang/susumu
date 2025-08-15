//! Language Server Protocol implementation for Susumu
//!
//! Provides IDE features like code completion, hover, diagnostics, etc.

use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse, Diagnostic,
    DiagnosticSeverity, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse,
    GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents, HoverParams,
    InitializeParams, Location, MarkedString, Position, Range, ServerCapabilities, SymbolKind,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit,
};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;

use crate::ast::{Expression, Program};
use crate::{Lexer, Parser, SusumuError};

pub struct SusumuLanguageServer {
    /// Stores the current state of open documents
    documents: HashMap<String, DocumentState>,
    /// Stores parsed ASTs for quick access
    ast_cache: HashMap<String, Program>,
    /// Stores function definitions for quick lookup
    function_defs: HashMap<String, FunctionInfo>,
}

struct DocumentState {
    content: String,
    version: i32,
    diagnostics: Vec<Diagnostic>,
}

struct FunctionInfo {
    name: String,
    params: Vec<String>,
    location: Location,
    documentation: Option<String>,
}

impl SusumuLanguageServer {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            ast_cache: HashMap::new(),
            function_defs: HashMap::new(),
        }
    }

    pub fn run(mut self) -> Result<(), Box<dyn Error + Sync + Send>> {
        eprintln!("Starting Susumu Language Server v0.2.9 with assignment support");

        let (connection, io_threads) = Connection::stdio();
        let server_capabilities = serde_json::to_value(&ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            completion_provider: Some(lsp_types::CompletionOptions {
                resolve_provider: Some(false),
                trigger_characters: Some(vec![
                    String::from("->"),
                    String::from("<-"),
                    String::from("("),
                    String::from("."),
                ]),
                ..Default::default()
            }),
            hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
            definition_provider: Some(lsp_types::OneOf::Left(true)),
            document_symbol_provider: Some(lsp_types::OneOf::Left(true)),
            document_formatting_provider: Some(lsp_types::OneOf::Left(true)),
            ..Default::default()
        })?;

        let initialization_params = connection.initialize(server_capabilities)?;
        let _params: InitializeParams = serde_json::from_value(initialization_params)?;

        eprintln!("Susumu LSP initialized");
        self.main_loop(connection)?;
        io_threads.join()?;

        eprintln!("Shutting down Susumu LSP");
        Ok(())
    }

    fn main_loop(&mut self, connection: Connection) -> Result<(), Box<dyn Error + Sync + Send>> {
        for msg in &connection.receiver {
            match msg {
                Message::Request(req) => {
                    if connection.handle_shutdown(&req)? {
                        return Ok(());
                    }
                    self.handle_request(req, &connection)?;
                }
                Message::Notification(not) => {
                    self.handle_notification(not, &connection)?;
                }
                Message::Response(_) => {}
            }
        }
        Ok(())
    }

    fn handle_request(
        &mut self,
        req: Request,
        connection: &Connection,
    ) -> Result<(), Box<dyn Error + Sync + Send>> {
        let result = match req.method.as_str() {
            "textDocument/completion" => {
                let params: CompletionParams = serde_json::from_value(req.params.clone())?;
                self.handle_completion(params)
            }
            "textDocument/hover" => {
                let params: HoverParams = serde_json::from_value(req.params.clone())?;
                self.handle_hover(params)
            }
            "textDocument/definition" => {
                let params: GotoDefinitionParams = serde_json::from_value(req.params.clone())?;
                self.handle_goto_definition(params)
            }
            "textDocument/documentSymbol" => {
                let params: DocumentSymbolParams = serde_json::from_value(req.params.clone())?;
                self.handle_document_symbols(params)
            }
            "textDocument/formatting" => {
                // Formatting not implemented yet
                Ok(serde_json::to_value(Vec::<TextEdit>::new())?)
            }
            _ => Ok(Value::Null),
        };

        let response = match result {
            Ok(result) => Response {
                id: req.id.clone(),
                result: Some(result),
                error: None,
            },
            Err(e) => Response {
                id: req.id.clone(),
                result: None,
                error: Some(lsp_server::ResponseError {
                    code: lsp_server::ErrorCode::InternalError as i32,
                    message: e.to_string(),
                    data: None,
                }),
            },
        };

        connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    fn handle_notification(
        &mut self,
        not: Notification,
        connection: &Connection,
    ) -> Result<(), Box<dyn Error + Sync + Send>> {
        match not.method.as_str() {
            "textDocument/didOpen" => {
                let params: DidOpenTextDocumentParams = serde_json::from_value(not.params)?;
                self.handle_did_open(params, connection)?;
            }
            "textDocument/didChange" => {
                let params: DidChangeTextDocumentParams = serde_json::from_value(not.params)?;
                self.handle_did_change(params, connection)?;
            }
            "textDocument/didSave" => {
                let params: DidSaveTextDocumentParams = serde_json::from_value(not.params)?;
                self.handle_did_save(params, connection)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_did_open(
        &mut self,
        params: DidOpenTextDocumentParams,
        connection: &Connection,
    ) -> Result<(), Box<dyn Error + Sync + Send>> {
        let uri = params.text_document.uri.to_string();
        let content = params.text_document.text;

        self.documents.insert(
            uri.clone(),
            DocumentState {
                content: content.clone(),
                version: params.text_document.version,
                diagnostics: Vec::new(),
            },
        );

        self.validate_document(&uri, &content, connection)?;
        Ok(())
    }

    fn handle_did_change(
        &mut self,
        params: DidChangeTextDocumentParams,
        connection: &Connection,
    ) -> Result<(), Box<dyn Error + Sync + Send>> {
        let uri = params.text_document.uri.to_string();

        if let Some(changes) = params.content_changes.first() {
            let content = changes.text.clone();

            if let Some(doc) = self.documents.get_mut(&uri) {
                doc.content = content.clone();
                doc.version = params.text_document.version;
            }

            self.validate_document(&uri, &content, connection)?;
        }
        Ok(())
    }

    fn handle_did_save(
        &mut self,
        params: DidSaveTextDocumentParams,
        connection: &Connection,
    ) -> Result<(), Box<dyn Error + Sync + Send>> {
        let uri = params.text_document.uri.to_string();

        if let Some(doc) = self.documents.get(&uri) {
            let content = doc.content.clone(); // Clone to avoid borrow conflicts
            self.validate_document(&uri, &content, connection)?;
        }
        Ok(())
    }

    fn validate_document(
        &mut self,
        uri: &str,
        content: &str,
        connection: &Connection,
    ) -> Result<(), Box<dyn Error + Sync + Send>> {
        let mut diagnostics = Vec::new();

        // Try to lex and parse the document
        match Lexer::new(content).tokenize() {
            Ok(tokens) => {
                match Parser::new(tokens).parse() {
                    Ok(ast) => {
                        // Cache the AST for later use
                        self.ast_cache.insert(uri.to_string(), ast.clone());

                        // Extract function definitions
                        self.extract_function_definitions(uri, &ast);

                        // Run semantic validation for undefined functions
                        self.validate_function_references(content, &ast, &mut diagnostics);
                    }
                    Err(e) => {
                        diagnostics.push(create_diagnostic_from_error(&e));
                    }
                }
            }
            Err(e) => {
                diagnostics.push(create_diagnostic_from_error(&e));
            }
        }

        // Update stored diagnostics
        if let Some(doc) = self.documents.get_mut(uri) {
            doc.diagnostics = diagnostics.clone();
        }

        // Send diagnostics to client
        let params = lsp_types::PublishDiagnosticsParams {
            uri: lsp_types::Url::parse(uri)?,
            diagnostics,
            version: None,
        };

        let notification = Notification {
            method: "textDocument/publishDiagnostics".to_string(),
            params: serde_json::to_value(params)?,
        };

        connection
            .sender
            .send(Message::Notification(notification))?;
        Ok(())
    }

    fn validate_function_references(
        &self,
        content: &str,
        ast: &Program,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let builtin_functions = get_builtin_function_names();
        let lines: Vec<&str> = content.lines().collect();

        // Collect all user-defined function names from this file and others
        let mut all_functions: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Add builtin functions
        for builtin in &builtin_functions {
            all_functions.insert(builtin.to_string());
        }

        // Add user-defined functions
        for (name, _) in &self.function_defs {
            all_functions.insert(name.clone());
        }

        // Add functions from current AST
        for function in &ast.functions {
            all_functions.insert(function.name.clone());
        }

        // Check function references in expressions
        if let Some(ref main_expr) = ast.main_expression {
            self.check_expression_for_undefined_functions(
                main_expr,
                &lines,
                &all_functions,
                diagnostics,
            );
        }

        // Check function references in function bodies
        for function in &ast.functions {
            self.check_expression_for_undefined_functions(
                &function.body,
                &lines,
                &all_functions,
                diagnostics,
            );
        }
    }

    fn check_expression_for_undefined_functions(
        &self,
        expr: &Expression,
        lines: &[&str],
        defined_functions: &std::collections::HashSet<String>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // Expression already imported at top

        match expr {
            Expression::FunctionCall { name, args } => {
                if !defined_functions.contains(name) {
                    // Find the line and column where this function is used
                    if let Some((line_num, col)) = self.find_identifier_in_content(lines, name) {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position {
                                    line: line_num as u32,
                                    character: col as u32,
                                },
                                end: Position {
                                    line: line_num as u32,
                                    character: (col + name.len()) as u32,
                                },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            source: Some("susumu".to_string()),
                            message: format!("Function '{name}' is not defined"),
                            ..Default::default()
                        });
                    }
                }

                // Recursively check arguments
                for arg in args {
                    self.check_expression_for_undefined_functions(
                        arg,
                        lines,
                        defined_functions,
                        diagnostics,
                    );
                }
            }
            Expression::ArrowChain { expressions, .. } => {
                for expr in expressions {
                    self.check_expression_for_undefined_functions(
                        expr,
                        lines,
                        defined_functions,
                        diagnostics,
                    );
                }
            }
            Expression::BinaryOp { left, right, .. } => {
                self.check_expression_for_undefined_functions(
                    left,
                    lines,
                    defined_functions,
                    diagnostics,
                );
                self.check_expression_for_undefined_functions(
                    right,
                    lines,
                    defined_functions,
                    diagnostics,
                );
            }
            Expression::Assignment { value, .. } => {
                self.check_expression_for_undefined_functions(
                    value,
                    lines,
                    defined_functions,
                    diagnostics,
                );
            }
            // Handle other expression types as needed
            _ => {}
        }
    }

    fn find_identifier_in_content(
        &self,
        lines: &[&str],
        identifier: &str,
    ) -> Option<(usize, usize)> {
        for (line_num, line) in lines.iter().enumerate() {
            if let Some(col) = line.find(identifier) {
                // Make sure it's a whole word, not part of another identifier
                let start_ok =
                    col == 0 || !line.chars().nth(col - 1).unwrap_or(' ').is_alphanumeric();
                let end_ok = col + identifier.len() >= line.len()
                    || !line
                        .chars()
                        .nth(col + identifier.len())
                        .unwrap_or(' ')
                        .is_alphanumeric();

                if start_ok && end_ok {
                    return Some((line_num, col));
                }
            }
        }
        None
    }

    fn extract_function_definitions(&mut self, uri: &str, ast: &Program) {
        // Clear old definitions for this file
        self.function_defs
            .retain(|_, info| !info.location.uri.as_str().starts_with(uri));

        // Extract new definitions from the functions field
        for function in &ast.functions {
            let info = FunctionInfo {
                name: function.name.clone(),
                params: function.params.iter().map(|p| p.name.clone()).collect(),
                location: Location {
                    uri: lsp_types::Url::parse(uri).unwrap(),
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        }, // TODO: Track actual positions
                        end: Position {
                            line: 0,
                            character: 0,
                        },
                    },
                },
                documentation: Some(format!(
                    "Function {} with {} parameters",
                    function.name,
                    function.params.len()
                )),
            };
            self.function_defs.insert(function.name.clone(), info);
        }
    }

    fn handle_completion(
        &self,
        _params: CompletionParams,
    ) -> Result<Value, Box<dyn Error + Sync + Send>> {
        let mut items = Vec::new();

        // Add builtin functions
        let builtin_functions = get_builtin_function_names();
        for name in &builtin_functions {
            items.push(CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(format!("Built-in function: {name}")),
                documentation: Some(lsp_types::Documentation::String(get_builtin_documentation(
                    name,
                ))),
                insert_text: Some(name.to_string()),
                ..Default::default()
            });
        }

        // Add user-defined functions
        for (name, info) in &self.function_defs {
            let params_str = info.params.join(", ");
            items.push(CompletionItem {
                label: name.clone(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(format!("{name}({params_str})")),
                documentation: info
                    .documentation
                    .as_ref()
                    .map(|doc| lsp_types::Documentation::String(doc.clone())),
                insert_text: Some(name.clone()),
                ..Default::default()
            });
        }

        // Add keywords
        for keyword in &[
            "return", "i", "e", "success", "error", "true", "false", "null",
        ] {
            items.push(CompletionItem {
                label: keyword.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(format!("Keyword: {keyword}")),
                insert_text: Some(keyword.to_string()),
                ..Default::default()
            });
        }

        // Add arrow operators
        items.push(CompletionItem {
            label: "->".to_string(),
            kind: Some(CompletionItemKind::OPERATOR),
            detail: Some("Forward arrow operator".to_string()),
            documentation: Some(lsp_types::Documentation::String(
                "Flows data forward to the next function".to_string(),
            )),
            insert_text: Some("-> ".to_string()),
            ..Default::default()
        });

        items.push(CompletionItem {
            label: "<-".to_string(),
            kind: Some(CompletionItemKind::OPERATOR),
            detail: Some("Backward arrow operator".to_string()),
            documentation: Some(lsp_types::Documentation::String(
                "Gathers data from the right into a function".to_string(),
            )),
            insert_text: Some(" <- ".to_string()),
            ..Default::default()
        });

        Ok(serde_json::to_value(CompletionResponse::Array(items))?)
    }

    fn handle_hover(&self, params: HoverParams) -> Result<Value, Box<dyn Error + Sync + Send>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        if let Some(doc) = self.documents.get(&uri) {
            // Get the word at the current position
            if let Some(word) = get_word_at_position(&doc.content, position) {
                // Check if it's a builtin function
                if get_builtin_function_names().contains(&word.as_str()) {
                    let hover = Hover {
                        contents: HoverContents::Scalar(MarkedString::String(format!(
                            "**{}** (built-in)\n\n{}",
                            word,
                            get_builtin_documentation(&word)
                        ))),
                        range: None,
                    };
                    return Ok(serde_json::to_value(hover)?);
                }

                // Check if it's a user-defined function
                if let Some(info) = self.function_defs.get(&word) {
                    let params_str = info.params.join(", ");
                    let hover = Hover {
                        contents: HoverContents::Scalar(MarkedString::String(format!(
                            "**{}({})** (user-defined)\n\n{}",
                            info.name,
                            params_str,
                            info.documentation
                                .as_deref()
                                .unwrap_or("No documentation available")
                        ))),
                        range: None,
                    };
                    return Ok(serde_json::to_value(hover)?);
                }
            }
        }

        Ok(Value::Null)
    }

    fn handle_goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Value, Box<dyn Error + Sync + Send>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        if let Some(doc) = self.documents.get(&uri) {
            if let Some(word) = get_word_at_position(&doc.content, position) {
                // Look for function definition
                if let Some(info) = self.function_defs.get(&word) {
                    return Ok(serde_json::to_value(GotoDefinitionResponse::Scalar(
                        info.location.clone(),
                    ))?);
                }
            }
        }

        Ok(Value::Null)
    }

    fn handle_document_symbols(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Value, Box<dyn Error + Sync + Send>> {
        let uri = params.text_document.uri.to_string();
        let mut symbols = Vec::new();

        if let Some(ast) = self.ast_cache.get(&uri) {
            for function in &ast.functions {
                let symbol = DocumentSymbol {
                    name: function.name.clone(),
                    detail: Some(format!(
                        "({})",
                        function
                            .params
                            .iter()
                            .map(|p| p.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )),
                    kind: SymbolKind::FUNCTION,
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: 0,
                            character: 0,
                        },
                    },
                    selection_range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: 0,
                            character: 0,
                        },
                    },
                    children: None,
                    tags: None,
                    deprecated: Some(false),
                };
                symbols.push(symbol);
            }
        }

        Ok(serde_json::to_value(DocumentSymbolResponse::Nested(
            symbols,
        ))?)
    }
}

fn create_diagnostic_from_error(error: &SusumuError) -> Diagnostic {
    let (line, col, message) = match error {
        SusumuError::LexerError {
            line,
            column,
            message,
        } => (*line, *column, message.clone()),
        SusumuError::ParserError { line, message } => (*line, 0, message.clone()),
        _ => (0, 0, error.to_string()),
    };

    Diagnostic {
        range: Range {
            start: Position {
                line: (line as u32).saturating_sub(1),
                character: (col as u32).saturating_sub(1),
            },
            end: Position {
                line: (line as u32).saturating_sub(1),
                character: col as u32,
            },
        },
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some("susumu".to_string()),
        message,
        ..Default::default()
    }
}

fn get_word_at_position(content: &str, position: Position) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();

    if let Some(line) = lines.get(position.line as usize) {
        let char_pos = position.character as usize;

        // Find word boundaries
        let mut start = char_pos;
        let mut end = char_pos;

        let chars: Vec<char> = line.chars().collect();

        // Find start of word
        while start > 0
            && chars
                .get(start - 1)
                .map(|c| c.is_alphanumeric() || *c == '_')
                .unwrap_or(false)
        {
            start -= 1;
        }

        // Find end of word
        while end < chars.len()
            && chars
                .get(end)
                .map(|c| c.is_alphanumeric() || *c == '_')
                .unwrap_or(false)
        {
            end += 1;
        }

        if start < end {
            return Some(chars[start..end].iter().collect());
        }
    }

    None
}

fn format_susumu_code(code: &str) -> String {
    // Simple formatter that ensures consistent spacing around arrows
    code.lines()
        .map(|line| {
            line.replace("->", " -> ")
                .replace("<-", " <- ")
                .replace("  ", " ") // Remove double spaces
                .trim()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn get_builtin_documentation(name: &str) -> String {
    match name {
        "add" => "Adds two or more numbers together\nExample: 5 -> add <- 3".to_string(),
        "subtract" => {
            "Subtracts the second number from the first\nExample: 10 -> subtract <- 3".to_string()
        }
        "multiply" => "Multiplies two or more numbers\nExample: 4 -> multiply <- 5".to_string(),
        "divide" => {
            "Divides the first number by the second\nExample: 20 -> divide <- 4".to_string()
        }
        "concat" => {
            "Concatenates strings together\nExample: \"Hello\" -> concat <- \" World\"".to_string()
        }
        "to_upper" => "Converts a string to uppercase\nExample: \"hello\" -> to_upper".to_string(),
        "to_lower" => "Converts a string to lowercase\nExample: \"HELLO\" -> to_lower".to_string(),
        "length" => {
            "Returns the length of a string or array\nExample: \"hello\" -> length".to_string()
        }
        "first" => "Returns the first element of an array\nExample: [1, 2, 3] -> first".to_string(),
        "last" => "Returns the last element of an array\nExample: [1, 2, 3] -> last".to_string(),
        "sort" => "Sorts an array in ascending order\nExample: [3, 1, 2] -> sort".to_string(),
        "reverse" => "Reverses an array or string\nExample: [1, 2, 3] -> reverse".to_string(),
        "print" => "Prints a value to the console\nExample: \"Hello\" -> print".to_string(),
        _ => format!("Built-in function: {name}"),
    }
}

fn get_builtin_function_names() -> Vec<&'static str> {
    vec![
        "add",
        "subtract",
        "multiply",
        "divide",
        "power",
        "sqrt",
        "abs",
        "min",
        "max",
        "concat",
        "to_upper",
        "to_lower",
        "length",
        "trim",
        "split",
        "first",
        "last",
        "sort",
        "reverse",
        "map",
        "filter",
        "reduce",
        "type_of",
        "to_string",
        "to_number",
        "print",
        "println",
    ]
}

pub fn run_lsp_server() -> Result<(), Box<dyn Error + Sync + Send>> {
    let server = SusumuLanguageServer::new();
    server.run()
}
