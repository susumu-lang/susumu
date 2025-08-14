import * as vscode from 'vscode';
import * as path from 'path';
import * as child_process from 'child_process';
import * as fs from 'fs';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
    // Extension is activating correctly
    console.log('Susumu extension is now active!');

    // Start the language server
    startLanguageServer(context);

    // Register commands
    registerCommands(context);

    // Register decorations and other features
    registerFeatures(context);
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

function startLanguageServer(context: vscode.ExtensionContext) {
    const config = vscode.workspace.getConfiguration('susumu');
    
    // Debug: Check if this function is called at all
    console.log('startLanguageServer called');
    
    // LSP enabled by default - we have a proper Rust LSP server
    if (!config.get('lsp.enabled', true)) {
        vscode.window.showWarningMessage('Susumu LSP is disabled in settings');
        return;
    }
    
    // Production: Use bundled LSP binary, fallback to user config
    const bundledServerPath = context.asAbsolutePath('bin/susumu-lsp');
    const userServerPath = config.get<string>('lsp.serverPath');
    
    // Use bundled binary if it exists, otherwise use user config
    let serverPath = bundledServerPath;
    if (!fs.existsSync(bundledServerPath) && userServerPath && userServerPath !== 'susumu-lsp') {
        serverPath = userServerPath;
    }
    
    // Check if the server binary exists
    if (!fs.existsSync(serverPath)) {
        console.error(`Susumu LSP server not found at: ${serverPath}`);
        vscode.window.showErrorMessage(
            `Susumu LSP server not found at: ${serverPath}. Language features will be limited.`,
            'Open Settings', 'Show Extension Folder'
        ).then(selection => {
            if (selection === 'Open Settings') {
                vscode.commands.executeCommand('workbench.action.openSettings', 'susumu.lsp.serverPath');
            } else if (selection === 'Show Extension Folder') {
                vscode.commands.executeCommand('revealFileInOS', vscode.Uri.file(context.extensionPath));
            }
        });
        return;
    }
    
    // LSP server found, proceeding to start

    const serverOptions: ServerOptions = {
        run: { command: serverPath, transport: TransportKind.stdio },
        debug: { command: serverPath, transport: TransportKind.stdio }
    };

    // Options to control the language client
    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'susumu' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.{susu,susumu}')
        }
    };

    try {
        // Create the language client and start it
        console.log('Creating LanguageClient...');
        client = new LanguageClient(
            'susumuLanguageServer',
            'Susumu Language Server',
            serverOptions,
            clientOptions
        );
        console.log('LanguageClient created successfully');
    } catch (error) {
        console.error('Failed to create LanguageClient:', error);
        vscode.window.showErrorMessage(`Failed to create LSP client: ${error}`);
        return;
    }

    // Start the client. This will also launch the server
    client.start().then(() => {
        vscode.window.showInformationMessage('Susumu Language Server is running');
    }).catch((error) => {
        console.error('Failed to start Susumu Language Server:', error);
        vscode.window.showErrorMessage(
            `Failed to start Susumu Language Server. Make sure '${serverPath}' is installed and in your PATH.`
        );
    });
}

function registerCommands(context: vscode.ExtensionContext) {
    // Show AST command
    const showASTCommand = vscode.commands.registerCommand('susumu.showAST', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.languageId !== 'susumu') {
            vscode.window.showErrorMessage('Please open a Susumu file first');
            return;
        }

        const text = editor.document.getText();
        try {
            // Call the Susumu interpreter to get the AST
            const result = await executeSusumuCode(text, { showAST: true });
            if (result.success) {
                // Show AST in a webview panel
                showASTPanel(context, text, result.ast || 'AST not available');
            } else {
                vscode.window.showErrorMessage(`Error parsing Susumu code: ${result.error}`);
            }
        } catch (error) {
            vscode.window.showErrorMessage(`Error parsing Susumu code: ${error}`);
        }
    });

    // Evaluate selection command
    const evaluateSelectionCommand = vscode.commands.registerCommand('susumu.evaluateSelection', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.languageId !== 'susumu') {
            vscode.window.showErrorMessage('Please open a Susumu file first');
            return;
        }

        const selection = editor.selection;
        const text = selection.isEmpty ? editor.document.getText() : editor.document.getText(selection);
        
        if (!text.trim()) {
            vscode.window.showErrorMessage('Please select some Susumu code to evaluate');
            return;
        }

        // Show output panel
        const outputChannel = vscode.window.createOutputChannel('Susumu Evaluation');
        outputChannel.show();
        outputChannel.appendLine(`Evaluating: ${text}`);
        
        try {
            // Call the Susumu interpreter
            const result = await executeSusumuCode(text, { debug: true });
            if (result.success) {
                outputChannel.appendLine(`Result: ${result.result}`);
                if (result.executionTime) {
                    outputChannel.appendLine(`Execution time: ${result.executionTime}ms`);
                }
                if (result.traces && result.traces.length > 0) {
                    outputChannel.appendLine('Execution traces:');
                    result.traces.forEach((trace: string) => {
                        outputChannel.appendLine(`  ${trace}`);
                    });
                }
            } else {
                outputChannel.appendLine(`Error: ${result.error}`);
                vscode.window.showErrorMessage(`Error evaluating Susumu code: ${result.error}`);
            }
            outputChannel.appendLine('');
        } catch (error) {
            outputChannel.appendLine(`Error: ${error}`);
            vscode.window.showErrorMessage(`Error evaluating Susumu code: ${error}`);
        }
    });

    // Run file command
    const runFileCommand = vscode.commands.registerCommand('susumu.runFile', async (uri?: vscode.Uri) => {
        let fileUri = uri;
        if (!fileUri) {
            const editor = vscode.window.activeTextEditor;
            if (!editor) {
                vscode.window.showErrorMessage('No Susumu file to run');
                return;
            }
            fileUri = editor.document.uri;
        }

        if (!fileUri.fsPath.endsWith('.susu') && !fileUri.fsPath.endsWith('.susumu')) {
            vscode.window.showErrorMessage('Please select a Susumu file (.susu or .susumu)');
            return;
        }

        const terminal = vscode.window.createTerminal('Susumu');
        terminal.show();
        terminal.sendText(`susumu "${fileUri.fsPath}"`);
    });

    context.subscriptions.push(showASTCommand, evaluateSelectionCommand, runFileCommand);
}

function registerFeatures(context: vscode.ExtensionContext) {
    // Register arrow operator decorations
    const arrowDecorationType = vscode.window.createTextEditorDecorationType({
        color: '#FF6B6B',
        fontWeight: 'bold'
    });

    // Update decorations when the active editor changes
    const updateDecorations = (editor: vscode.TextEditor) => {
        if (!editor || editor.document.languageId !== 'susumu') {
            return;
        }

        const text = editor.document.getText();
        const decorations: vscode.DecorationOptions[] = [];
        const arrowRegex = /(->|<-)/g;
        let match;

        while ((match = arrowRegex.exec(text)) !== null) {
            const startPos = editor.document.positionAt(match.index);
            const endPos = editor.document.positionAt(match.index + match[0].length);
            
            decorations.push({
                range: new vscode.Range(startPos, endPos),
                hoverMessage: match[0] === '->' 
                    ? 'Forward arrow: flows data to the next function' 
                    : 'Backward arrow: gathers data from the right'
            });
        }

        editor.setDecorations(arrowDecorationType, decorations);
    };

    // Update decorations for the current editor
    if (vscode.window.activeTextEditor) {
        updateDecorations(vscode.window.activeTextEditor);
    }

    // Update decorations when the active editor changes
    vscode.window.onDidChangeActiveTextEditor((editor) => {
        if (editor) {
            updateDecorations(editor);
        }
    }, null, context.subscriptions);

    // Update decorations when the document changes
    vscode.workspace.onDidChangeTextDocument(event => {
        const editor = vscode.window.activeTextEditor;
        if (editor && event.document === editor.document) {
            updateDecorations(editor);
        }
    }, null, context.subscriptions);

    // Status bar item to show Susumu version
    const statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    statusBarItem.text = "$(arrow-right) Susumu";
    statusBarItem.tooltip = "Susumu Language Support";
    statusBarItem.command = 'susumu.showAST';
    
    const updateStatusBar = () => {
        const editor = vscode.window.activeTextEditor;
        if (editor && editor.document.languageId === 'susumu') {
            statusBarItem.show();
        } else {
            statusBarItem.hide();
        }
    };

    updateStatusBar();
    vscode.window.onDidChangeActiveTextEditor(updateStatusBar, null, context.subscriptions);

    context.subscriptions.push(statusBarItem);

    // Code lens provider for functions
    const codeLensProvider = new SusumuCodeLensProvider();
    vscode.languages.registerCodeLensProvider('susumu', codeLensProvider);
    
    // Completion provider for IntelliSense/typeahead
    const completionProvider = new SusumuCompletionProvider();
    vscode.languages.registerCompletionItemProvider('susumu', completionProvider, '->', '<-', '@');
    
    // Hover provider for documentation
    const hoverProvider = new SusumuHoverProvider();
    vscode.languages.registerHoverProvider('susumu', hoverProvider);
    
    // Document symbol provider for outline
    const symbolProvider = new SusumuDocumentSymbolProvider();
    vscode.languages.registerDocumentSymbolProvider('susumu', symbolProvider);
    
    // Signature help provider for function parameters
    const signatureProvider = new SusumuSignatureHelpProvider();
    vscode.languages.registerSignatureHelpProvider('susumu', signatureProvider, '(', ',');
    
    // Diagnostics handled by LSP server - no local diagnostics needed
    // const diagnosticCollection = vscode.languages.createDiagnosticCollection('susumu');
    // context.subscriptions.push(diagnosticCollection);
    
    // Diagnostics disabled - handled by LSP server
    const updateDiagnostics = (document: vscode.TextDocument) => {
        // LSP server handles all diagnostics
        return;
        
        const diagnostics: vscode.Diagnostic[] = [];
        const text = document.getText();
        const lines = text.split('\n');
        
        lines.forEach((line, lineNum) => {
            // Check for common syntax errors
            
            // Unmatched braces
            const openBraces = (line.match(/{/g) || []).length;
            const closeBraces = (line.match(/}/g) || []).length;
            if (openBraces > 0 || closeBraces > 0) {
                // This is a simple check - in a real implementation you'd track brace balance across the file
                if (line.includes('{') && !line.includes('}') && !lines[lineNum + 1]?.includes('}')) {
                    // Potential unclosed brace (this is a simplified check)
                }
            }
            
            // Invalid arrow patterns (be more specific to avoid false positives)
            if (line.includes('->-') || line.includes('<--')) {
                const match = line.match(/(->-|<--)/);
                if (match) {
                    const range = new vscode.Range(lineNum, match.index!, lineNum, match.index! + match[0].length);
                    diagnostics.push(new vscode.Diagnostic(
                        range,
                        `Invalid arrow operator: ${match[0]}. Use -> or <-`,
                        vscode.DiagnosticSeverity.Error
                    ));
                }
            }
            
            // Missing arrow in function calls (only for very simple cases to avoid false positives)
            const invalidFunctionCall = line.match(/^(\w+)\s*\(\s*(\w+)\s*\)\s*$/);
            if (invalidFunctionCall && !line.includes('//')) {
                const range = new vscode.Range(lineNum, 0, lineNum, line.length);
                diagnostics.push(new vscode.Diagnostic(
                    range,
                    `Consider using arrow syntax: ${invalidFunctionCall[2]} -> ${invalidFunctionCall[1]}`,
                    vscode.DiagnosticSeverity.Information
                ));
            }
            
            // Undefined variables (more conservative check to avoid false positives)
            const scopeInfo = analyzeScope(document, new vscode.Position(lineNum, line.length));
            
            // Only check variables that are clearly being used in arrow flows
            const usagePattern = /\b([a-zA-Z_][a-zA-Z0-9_]*)\s*->/g;
            let match;
            while ((match = usagePattern.exec(line)) !== null) {
                const variable = match[1];
                const keywords = ['i', 'ei', 'e', 'fe', 'w', 'return', 'error', 'main', 'some', 'none',
                                'success', 'valid', 'positive', 'negative', 'zero', 'empty', 'found', 'match',
                                'userId', 'orderData', 'user', 'receipt', 'errorMsg', 'result', 'validUser',
                                'paymentError', 'primaryError', 'fallbackResult', 'fallbackError', 'successMsg',
                                'testOrder'];
                const builtins = ['add', 'subtract', 'multiply', 'divide', 'print', 'length', 'first', 'last',
                                'validateOrder', 'enrichWithDefaults', 'calculateTotals', 'processPayment',
                                'sendConfirmation', 'updateInventory', 'logSuccess', 'logError', 'sendFailureNotification',
                                'initiateRefund', 'validateUserId', 'lookupInDatabase', 'prepareOrder', 'combineUserOrder',
                                'notifyUser', 'updateUserHistory', 'notifyUserOfFailure', 'logInvalidUser',
                                'simplifiedProcessing', 'logPrimaryFailure', 'logCriticalFailure', 'notifyAdmins',
                                'multiply'];
                
                // Only flag if it's clearly undefined and not a common pattern
                if (!keywords.includes(variable) && 
                    !builtins.includes(variable) &&
                    !scopeInfo.variables.has(variable) && 
                    !scopeInfo.functions.has(variable) &&
                    !scopeInfo.parameters.has(variable) &&
                    !line.includes('=') && // Don't flag assignments
                    !line.includes('match') && // Don't flag match expressions
                    !line.includes('{') && // Don't flag object literals
                    variable.length > 2) { // Only flag longer variable names
                    
                    const range = new vscode.Range(lineNum, match.index!, lineNum, match.index! + variable.length);
                    diagnostics.push(new vscode.Diagnostic(
                        range,
                        `Variable '${variable}' may not be defined`,
                        vscode.DiagnosticSeverity.Hint // Changed to Hint instead of Warning
                    ));
                }
            }
        });
        
        // diagnosticCollection.set(document.uri, diagnostics); // Handled by LSP
    };
    
    // Update diagnostics on document changes
    vscode.workspace.onDidChangeTextDocument(event => {
        updateDiagnostics(event.document);
    }, null, context.subscriptions);
    
    vscode.workspace.onDidOpenTextDocument(updateDiagnostics, null, context.subscriptions);
    
    // Update diagnostics for already open documents
    vscode.workspace.textDocuments.forEach(updateDiagnostics);
}

class SusumuCodeLensProvider implements vscode.CodeLensProvider {
    provideCodeLenses(document: vscode.TextDocument): vscode.CodeLens[] {
        const codeLenses: vscode.CodeLens[] = [];
        const text = document.getText();
        const lines = text.split('\n');

        for (let i = 0; i < lines.length; i++) {
            const line = lines[i];
            const functionMatch = line.match(/^\s*(\w+)\s*\(/);
            
            if (functionMatch) {
                const range = new vscode.Range(i, 0, i, line.length);
                
                // Add "Run Function" code lens
                codeLenses.push(new vscode.CodeLens(range, {
                    title: "▶ Run Function",
                    command: 'susumu.evaluateSelection',
                    arguments: [range]
                }));

                // Add "Show Documentation" code lens
                codeLenses.push(new vscode.CodeLens(range, {
                    title: "📖 Show Docs",
                    command: 'susumu.showAST',
                    arguments: [functionMatch[1]]
                }));
            }
        }

        return codeLenses;
    }
}

// Scope analysis for variables and functions
interface ScopeInfo {
    variables: Set<string>;
    functions: Map<string, { params: string[], line: number }>;
    parameters: Set<string>;
}

function analyzeScope(document: vscode.TextDocument, position: vscode.Position): ScopeInfo {
    const scope: ScopeInfo = {
        variables: new Set(),
        functions: new Map(),
        parameters: new Set()
    };
    
    const text = document.getText();
    const lines = text.split('\n');
    
    // Find current function context
    let currentFunction: string | null = null;
    let braceDepth = 0;
    let inCurrentFunction = false;
    
    for (let i = 0; i <= position.line; i++) {
        const line = lines[i];
        
        // Track function definitions
        const functionMatch = line.match(/^(\s*)([a-zA-Z_][a-zA-Z0-9_]*)\s*\(([^)]*)\)\s*\{/);
        if (functionMatch) {
            const funcName = functionMatch[2];
            const params = functionMatch[3].split(',').map(p => p.trim()).filter(p => p);
            scope.functions.set(funcName, { params, line: i });
            
            if (i <= position.line) {
                currentFunction = funcName;
                inCurrentFunction = true;
                braceDepth = 1;
                
                // Add parameters to scope if we're in this function
                params.forEach(param => {
                    if (param) scope.parameters.add(param);
                });
            }
        }
        
        // Track brace depth to determine if we're still in the function
        if (inCurrentFunction && i > 0) {
            for (const char of line) {
                if (char === '{') braceDepth++;
                if (char === '}') braceDepth--;
                if (braceDepth === 0) {
                    inCurrentFunction = false;
                    currentFunction = null;
                    scope.parameters.clear();
                    break;
                }
            }
        }
        
        // Find variable assignments and arrow targets
        if (i <= position.line) {
            // Pattern: variable -> something or something -> variable
            const arrowMatches = line.matchAll(/([a-zA-Z_][a-zA-Z0-9_]*)\s*->/g);
            for (const match of arrowMatches) {
                scope.variables.add(match[1]);
            }
            
            // Pattern: something <- variable
            const backArrowMatches = line.matchAll(/<-\s*([a-zA-Z_][a-zA-Z0-9_]*)/g);
            for (const match of backArrowMatches) {
                scope.variables.add(match[1]);
            }
            
            // Pattern: variable = something (assignments)
            const assignMatches = line.matchAll(/([a-zA-Z_][a-zA-Z0-9_]*)\s*=/g);
            for (const match of assignMatches) {
                scope.variables.add(match[1]);
            }
            
            // Pattern: return <- variable
            const returnMatches = line.matchAll(/return\s*<-\s*([a-zA-Z_][a-zA-Z0-9_]*)/g);
            for (const match of returnMatches) {
                scope.variables.add(match[1]);
            }
            
            // Pattern: foreach loops - fe item in collection
            const foreachMatches = line.matchAll(/fe\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+in\s+([a-zA-Z_][a-zA-Z0-9_]*)/g);
            for (const match of foreachMatches) {
                scope.variables.add(match[1]); // loop variable
                scope.variables.add(match[2]); // collection
            }
            
            // Pattern: Conditional variable patterns - value -> i condition
            const conditionalMatches = line.matchAll(/([a-zA-Z_][a-zA-Z0-9_]*)\s*->\s*i\s+/g);
            for (const match of conditionalMatches) {
                scope.variables.add(match[1]);
            }
            
            // Pattern: Match patterns - some <- variable
            const someMatches = line.matchAll(/some\s*<-\s*([a-zA-Z_][a-zA-Z0-9_]*)/g);
            for (const match of someMatches) {
                scope.variables.add(match[1]);
            }
            
            // Pattern: Error patterns - error <- variable
            const errorMatches = line.matchAll(/error\s*<-\s*([a-zA-Z_][a-zA-Z0-9_]*)/g);
            for (const match of errorMatches) {
                scope.variables.add(match[1]);
            }
            
            // Pattern: Match expression bindings - success <- result, error <- message
            const matchBindings = line.matchAll(/([a-zA-Z_][a-zA-Z0-9_]*)\s*<-\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*->/g);
            for (const match of matchBindings) {
                scope.variables.add(match[1]); // pattern (success, error, etc.)
                scope.variables.add(match[2]); // bound variable (result, message, etc.)
            }
            
            // Pattern: Simple assignments like testOrder = {...}
            const simpleAssignments = line.matchAll(/([a-zA-Z_][a-zA-Z0-9_]*)\s*=/g);
            for (const match of simpleAssignments) {
                scope.variables.add(match[1]);
            }
            
            // Pattern: Variables in property access (simplified) - user.name, user.id
            const propertyAccess = line.matchAll(/([a-zA-Z_][a-zA-Z0-9_]*)\./g);
            for (const match of propertyAccess) {
                scope.variables.add(match[1]);
            }
            
            // Pattern: Variables in string concatenation
            const stringConcat = line.matchAll(/"[^"]*"\s*\+\s*([a-zA-Z_][a-zA-Z0-9_]*)/g);
            for (const match of stringConcat) {
                scope.variables.add(match[1]);
            }
            
            // Pattern: Tuple/parentheses groupings - (user, order)
            const tupleVars = line.matchAll(/\(\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*,\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\)/g);
            for (const match of tupleVars) {
                scope.variables.add(match[1]);
                scope.variables.add(match[2]);
            }
        }
    }
    
    return scope;
}

// Completion provider for IntelliSense
class SusumuCompletionProvider implements vscode.CompletionItemProvider {
    provideCompletionItems(
        document: vscode.TextDocument,
        position: vscode.Position,
        token: vscode.CancellationToken,
        context: vscode.CompletionContext
    ): vscode.CompletionItem[] {
        const completions: vscode.CompletionItem[] = [];
        const line = document.lineAt(position).text;
        const linePrefix = line.slice(0, position.character);
        
        // Analyze current scope for variables and functions
        const scopeInfo = analyzeScope(document, position);

        // Add user-defined functions from scope
        scopeInfo.functions.forEach((funcInfo, funcName) => {
            const completion = new vscode.CompletionItem(funcName, vscode.CompletionItemKind.Function);
            completion.detail = `${funcName}(${funcInfo.params.join(', ')})`;
            completion.documentation = `User-defined function (line ${funcInfo.line + 1})`;
            completion.insertText = funcName;
            completion.sortText = '0' + funcName; // Prioritize user functions
            completions.push(completion);
        });

        // Add variables from scope
        scopeInfo.variables.forEach(varName => {
            // Skip built-in function names
            const builtinNames = ['add', 'subtract', 'multiply', 'divide', 'print', 'length', 'first', 'last'];
            if (!builtinNames.includes(varName) && !scopeInfo.functions.has(varName)) {
                const completion = new vscode.CompletionItem(varName, vscode.CompletionItemKind.Variable);
                completion.detail = 'Variable in scope';
                completion.documentation = `Variable: ${varName}`;
                completion.insertText = varName;
                completion.sortText = '1' + varName; // Second priority after functions
                completions.push(completion);
            }
        });

        // Add function parameters from current scope
        scopeInfo.parameters.forEach(paramName => {
            if (!scopeInfo.variables.has(paramName)) { // Don't duplicate if already added as variable
                const completion = new vscode.CompletionItem(paramName, vscode.CompletionItemKind.Property);
                completion.detail = 'Function parameter';
                completion.documentation = `Parameter: ${paramName}`;
                completion.insertText = paramName;
                completion.sortText = '0' + paramName; // High priority like functions
                completions.push(completion);
            }
        });

        // Arrow operators
        if (linePrefix.endsWith('-') || context.triggerCharacter === '->') {
            const arrowCompletion = new vscode.CompletionItem('->', vscode.CompletionItemKind.Operator);
            arrowCompletion.detail = 'Forward arrow operator';
            arrowCompletion.documentation = 'Flows data from left to right';
            arrowCompletion.insertText = '->';
            completions.push(arrowCompletion);
        }

        if (linePrefix.endsWith('<') || context.triggerCharacter === '<-') {
            const backwardArrowCompletion = new vscode.CompletionItem('<-', vscode.CompletionItemKind.Operator);
            backwardArrowCompletion.detail = 'Backward arrow operator';
            backwardArrowCompletion.documentation = 'Gathers data from the right';
            backwardArrowCompletion.insertText = '<-';
            completions.push(backwardArrowCompletion);
        }

        // Annotations
        if (linePrefix.endsWith('@') || context.triggerCharacter === '@') {
            const annotations = [
                { name: '@trace', detail: 'Trace annotation', doc: 'Add execution tracing for debugging' },
                { name: '@monitor', detail: 'Monitor annotation', doc: 'Add performance monitoring' },
                { name: '@config', detail: 'Config annotation', doc: 'Add configuration metadata' },
                { name: '@parallel', detail: 'Parallel annotation', doc: 'Mark for parallel execution' },
                { name: '@debug', detail: 'Debug annotation', doc: 'Add debug information' }
            ];

            annotations.forEach(ann => {
                const completion = new vscode.CompletionItem(ann.name, vscode.CompletionItemKind.Function);
                completion.detail = ann.detail;
                completion.documentation = ann.doc;
                completion.insertText = new vscode.SnippetString(`${ann.name} <- $1`);
                completions.push(completion);
            });
        }

        // Built-in functions
        const builtins = [
            // Math functions
            { name: 'add', params: '(x, y)', doc: 'Add two numbers' },
            { name: 'subtract', params: '(x, y)', doc: 'Subtract second number from first' },
            { name: 'multiply', params: '(x, y)', doc: 'Multiply two numbers' },
            { name: 'divide', params: '(x, y)', doc: 'Divide first number by second' },
            { name: 'power', params: '(base, exp)', doc: 'Raise base to the power of exponent' },
            { name: 'sqrt', params: '(x)', doc: 'Square root of number' },
            { name: 'abs', params: '(x)', doc: 'Absolute value' },
            { name: 'min', params: '(x, y)', doc: 'Minimum of two numbers' },
            { name: 'max', params: '(x, y)', doc: 'Maximum of two numbers' },
            
            // String functions
            { name: 'concat', params: '(str1, str2)', doc: 'Concatenate two strings' },
            { name: 'length', params: '(str)', doc: 'Get string length' },
            { name: 'to_upper', params: '(str)', doc: 'Convert to uppercase' },
            { name: 'to_lower', params: '(str)', doc: 'Convert to lowercase' },
            { name: 'trim', params: '(str)', doc: 'Remove whitespace' },
            { name: 'split', params: '(str, delimiter)', doc: 'Split string by delimiter' },
            
            // Array functions
            { name: 'first', params: '(array)', doc: 'Get first element' },
            { name: 'last', params: '(array)', doc: 'Get last element' },
            { name: 'sort', params: '(array)', doc: 'Sort array elements' },
            { name: 'reverse', params: '(array)', doc: 'Reverse array order' },
            { name: 'map', params: '(array, fn)', doc: 'Transform each element' },
            { name: 'filter', params: '(array, predicate)', doc: 'Filter elements' },
            { name: 'reduce', params: '(array, fn, initial)', doc: 'Reduce to single value' },
            
            // Type functions
            { name: 'type_of', params: '(value)', doc: 'Get type of value' },
            { name: 'to_string', params: '(value)', doc: 'Convert to string' },
            { name: 'to_number', params: '(value)', doc: 'Convert to number' },
            
            // IO functions
            { name: 'print', params: '(value)', doc: 'Print value to console' },
            { name: 'println', params: '(value)', doc: 'Print value with newline' },
            { name: 'read', params: '()', doc: 'Read input from user' }
        ];

        builtins.forEach(builtin => {
            const completion = new vscode.CompletionItem(builtin.name, vscode.CompletionItemKind.Function);
            completion.detail = `${builtin.name}${builtin.params}`;
            completion.documentation = builtin.doc;
            completion.insertText = builtin.name;
            completions.push(completion);
        });

        // Keywords
        const keywords = [
            { name: 'i', doc: 'Conditional if statement' },
            { name: 'ei', doc: 'Else-if branch' },
            { name: 'e', doc: 'Else branch' },
            { name: 'fe', doc: 'Foreach loop' },
            { name: 'w', doc: 'While loop' },
            { name: 'return', doc: 'Return statement' },
            { name: 'error', doc: 'Error statement' },
            { name: 'main', doc: 'Main function entry point' }
        ];

        keywords.forEach(keyword => {
            const completion = new vscode.CompletionItem(keyword.name, vscode.CompletionItemKind.Keyword);
            completion.documentation = keyword.doc;
            completion.insertText = keyword.name;
            completions.push(completion);
        });

        // Condition types
        const conditions = [
            { name: 'success', doc: 'True if value is not null/false' },
            { name: 'error', doc: 'True if value indicates an error' },
            { name: 'valid', doc: 'True if value is valid' },
            { name: 'positive', doc: 'True if number is positive' },
            { name: 'negative', doc: 'True if number is negative' },
            { name: 'zero', doc: 'True if number is zero' },
            { name: 'empty', doc: 'True if collection is empty' },
            { name: 'found', doc: 'True if item was found' }
        ];

        conditions.forEach(condition => {
            const completion = new vscode.CompletionItem(condition.name, vscode.CompletionItemKind.EnumMember);
            completion.documentation = condition.doc;
            completion.insertText = condition.name;
            completions.push(completion);
        });

        return completions;
    }
}

// Hover provider for documentation
class SusumuHoverProvider implements vscode.HoverProvider {
    provideHover(
        document: vscode.TextDocument,
        position: vscode.Position,
        token: vscode.CancellationToken
    ): vscode.Hover | undefined {
        const wordRange = document.getWordRangeAtPosition(position);
        if (!wordRange) {
            return;
        }

        const word = document.getText(wordRange);
        const line = document.lineAt(position.line).text;
        
        // Analyze scope for user-defined items
        const scopeInfo = analyzeScope(document, position);

        // Check for user-defined functions
        if (scopeInfo.functions.has(word)) {
            const funcInfo = scopeInfo.functions.get(word)!;
            return new vscode.Hover(
                new vscode.MarkdownString(`**${word}** - User-defined function\n\nParameters: \`(${funcInfo.params.join(', ')})\`\n\nDefined on line ${funcInfo.line + 1}`),
                wordRange
            );
        }

        // Check for variables in scope
        if (scopeInfo.variables.has(word)) {
            return new vscode.Hover(
                new vscode.MarkdownString(`**${word}** - Variable in scope\n\nThis variable is available in the current context.`),
                wordRange
            );
        }

        // Check for function parameters
        if (scopeInfo.parameters.has(word)) {
            return new vscode.Hover(
                new vscode.MarkdownString(`**${word}** - Function parameter\n\nThis is a parameter of the current function.`),
                wordRange
            );
        }

        // Check for arrow operators
        if (line.includes('->')) {
            const arrowRange = document.getWordRangeAtPosition(position, /->/);
            if (arrowRange) {
                return new vscode.Hover(
                    new vscode.MarkdownString(`**Forward Arrow (→)** \n\nFlows data from left to right.\n\n\`\`\`susumu\n5 -> add <- 3  // 5 flows to add function\n\`\`\``),
                    arrowRange
                );
            }
        }

        if (line.includes('<-')) {
            const arrowRange = document.getWordRangeAtPosition(position, /<-/);
            if (arrowRange) {
                return new vscode.Hover(
                    new vscode.MarkdownString(`**Backward Arrow (←)** \n\nGathers data from the right side.\n\n\`\`\`susumu\n5 -> add <- 3  // 3 flows in from right\n\`\`\``),
                    arrowRange
                );
            }
        }

        // Function documentation
        const functionDocs: { [key: string]: string } = {
            'add': '**add(x, y)** - Add two numbers\n\n```susumu\n5 -> add <- 3  // Result: 8\n```',
            'subtract': '**subtract(x, y)** - Subtract second from first\n\n```susumu\n10 -> subtract <- 3  // Result: 7\n```',
            'multiply': '**multiply(x, y)** - Multiply two numbers\n\n```susumu\n4 -> multiply <- 5  // Result: 20\n```',
            'divide': '**divide(x, y)** - Divide first by second\n\n```susumu\n15 -> divide <- 3  // Result: 5\n```',
            'print': '**print(value)** - Print value to console\n\n```susumu\n"Hello" -> print  // Output: Hello\n```',
            'first': '**first(array)** - Get first element\n\n```susumu\n[1, 2, 3] -> first  // Result: 1\n```',
            'last': '**last(array)** - Get last element\n\n```susumu\n[1, 2, 3] -> last  // Result: 3\n```',
            'length': '**length(str/array)** - Get length\n\n```susumu\n"hello" -> length  // Result: 5\n[1, 2, 3] -> length  // Result: 3\n```'
        };

        if (functionDocs[word]) {
            return new vscode.Hover(
                new vscode.MarkdownString(functionDocs[word]),
                wordRange
            );
        }

        // Keyword documentation
        const keywordDocs: { [key: string]: string } = {
            'i': '**i** - Conditional if statement\n\n```susumu\nvalue -> i success {\n    // success branch\n} e {\n    // else branch\n}\n```',
            'ei': '**ei** - Else-if branch\n\n```susumu\nvalue -> i positive {\n    // positive\n} ei zero {\n    // zero\n} e {\n    // negative\n}\n```',
            'e': '**e** - Else branch\n\n```susumu\nvalue -> i success {\n    // if branch\n} e {\n    // else branch\n}\n```'
        };

        if (keywordDocs[word]) {
            return new vscode.Hover(
                new vscode.MarkdownString(keywordDocs[word]),
                wordRange
            );
        }

        return undefined;
    }
}

// Signature help provider for function parameters
class SusumuSignatureHelpProvider implements vscode.SignatureHelpProvider {
    provideSignatureHelp(
        document: vscode.TextDocument,
        position: vscode.Position,
        token: vscode.CancellationToken,
        context: vscode.SignatureHelpContext
    ): vscode.SignatureHelp | undefined {
        const line = document.lineAt(position).text;
        const linePrefix = line.slice(0, position.character);
        
        // Find function call pattern
        const functionMatch = linePrefix.match(/([a-zA-Z_][a-zA-Z0-9_]*)\s*\([^)]*$/);
        if (!functionMatch) {
            return undefined;
        }
        
        const functionName = functionMatch[1];
        const scopeInfo = analyzeScope(document, position);
        
        const signatureHelp = new vscode.SignatureHelp();
        
        // Check user-defined functions first
        if (scopeInfo.functions.has(functionName)) {
            const funcInfo = scopeInfo.functions.get(functionName)!;
            const signature = new vscode.SignatureInformation(`${functionName}(${funcInfo.params.join(', ')})`);
            
            funcInfo.params.forEach(param => {
                signature.parameters.push(new vscode.ParameterInformation(param));
            });
            
            signatureHelp.signatures.push(signature);
            signatureHelp.activeSignature = 0;
            
            // Determine active parameter based on commas
            const parameterIndex = (linePrefix.match(/,/g) || []).length;
            signatureHelp.activeParameter = Math.min(parameterIndex, funcInfo.params.length - 1);
            
            return signatureHelp;
        }
        
        // Built-in functions
        const builtinSignatures: { [key: string]: { params: string[], description: string } } = {
            'add': { params: ['x', 'y'], description: 'Add two numbers' },
            'subtract': { params: ['x', 'y'], description: 'Subtract second from first' },
            'multiply': { params: ['x', 'y'], description: 'Multiply two numbers' },
            'divide': { params: ['x', 'y'], description: 'Divide first by second' },
            'power': { params: ['base', 'exponent'], description: 'Raise base to power' },
            'sqrt': { params: ['x'], description: 'Square root' },
            'abs': { params: ['x'], description: 'Absolute value' },
            'min': { params: ['x', 'y'], description: 'Minimum of two numbers' },
            'max': { params: ['x', 'y'], description: 'Maximum of two numbers' },
            'concat': { params: ['str1', 'str2'], description: 'Concatenate strings' },
            'length': { params: ['value'], description: 'Get length' },
            'to_upper': { params: ['str'], description: 'Convert to uppercase' },
            'to_lower': { params: ['str'], description: 'Convert to lowercase' },
            'trim': { params: ['str'], description: 'Remove whitespace' },
            'split': { params: ['str', 'delimiter'], description: 'Split string' },
            'first': { params: ['array'], description: 'Get first element' },
            'last': { params: ['array'], description: 'Get last element' },
            'sort': { params: ['array'], description: 'Sort array' },
            'reverse': { params: ['array'], description: 'Reverse array' },
            'map': { params: ['array', 'function'], description: 'Transform elements' },
            'filter': { params: ['array', 'predicate'], description: 'Filter elements' },
            'reduce': { params: ['array', 'function', 'initial'], description: 'Reduce to single value' },
            'type_of': { params: ['value'], description: 'Get type' },
            'to_string': { params: ['value'], description: 'Convert to string' },
            'to_number': { params: ['value'], description: 'Convert to number' },
            'print': { params: ['value'], description: 'Print to console' },
            'println': { params: ['value'], description: 'Print with newline' },
            'read': { params: [], description: 'Read user input' }
        };
        
        if (builtinSignatures[functionName]) {
            const builtin = builtinSignatures[functionName];
            const signature = new vscode.SignatureInformation(`${functionName}(${builtin.params.join(', ')})`);
            signature.documentation = builtin.description;
            
            builtin.params.forEach(param => {
                signature.parameters.push(new vscode.ParameterInformation(param));
            });
            
            signatureHelp.signatures.push(signature);
            signatureHelp.activeSignature = 0;
            
            // Determine active parameter based on commas
            const parameterIndex = (linePrefix.match(/,/g) || []).length;
            signatureHelp.activeParameter = Math.min(parameterIndex, builtin.params.length - 1);
            
            return signatureHelp;
        }
        
        return undefined;
    }
}

// Document symbol provider
class SusumuDocumentSymbolProvider implements vscode.DocumentSymbolProvider {
    provideDocumentSymbols(document: vscode.TextDocument): vscode.DocumentSymbol[] {
        const symbols: vscode.DocumentSymbol[] = [];
        const text = document.getText();
        const lines = text.split('\n');

        for (let i = 0; i < lines.length; i++) {
            const line = lines[i];
            const functionMatch = line.match(/^\s*(\w+)\s*\(([^)]*)\)/);
            
            if (functionMatch) {
                const name = functionMatch[1];
                const params = functionMatch[2];
                const range = new vscode.Range(i, 0, i, line.length);
                
                const symbol = new vscode.DocumentSymbol(
                    name,
                    params ? `(${params})` : '()',
                    vscode.SymbolKind.Function,
                    range,
                    range
                );
                
                symbols.push(symbol);
            }
        }

        return symbols;
    }
}

// Susumu execution interface
interface SusumuExecutionOptions {
    debug?: boolean;
    showAST?: boolean;
}

interface SusumuExecutionResult {
    success: boolean;
    result?: string;
    error?: string;
    executionTime?: number;
    traces?: string[];
    ast?: string;
}

// Execute Susumu code using the Rust interpreter
async function executeSusumuCode(code: string, options: SusumuExecutionOptions = {}): Promise<SusumuExecutionResult> {
    return new Promise((resolve) => {
        const config = vscode.workspace.getConfiguration('susumu');
        const interpreterPath = config.get<string>('interpreter.path') || 'susumu';
        
        // Build command arguments
        const args: string[] = [];
        if (options.debug) {
            args.push('--debug');
        }
        if (options.showAST) {
            args.push('--ast');
        }
        
        // Execute the interpreter
        const child = child_process.spawn(interpreterPath, args, {
            stdio: ['pipe', 'pipe', 'pipe']
        });
        
        let stdout = '';
        let stderr = '';
        
        child.stdout.on('data', (data) => {
            stdout += data.toString();
        });
        
        child.stderr.on('data', (data) => {
            stderr += data.toString();
        });
        
        child.on('close', (code) => {
            if (code === 0) {
                try {
                    // Try to parse JSON output for debug info
                    if (options.debug && stdout.includes('{')) {
                        const jsonMatch = stdout.match(/\{.*\}/s);
                        if (jsonMatch) {
                            const debugInfo = JSON.parse(jsonMatch[0]);
                            resolve({
                                success: true,
                                result: debugInfo.result || stdout.trim(),
                                executionTime: debugInfo.execution_time_ms,
                                traces: debugInfo.execution_traces
                            });
                            return;
                        }
                    }
                    
                    // Regular output
                    resolve({
                        success: true,
                        result: stdout.trim(),
                        ast: options.showAST ? stdout : undefined
                    });
                } catch (parseError) {
                    resolve({
                        success: true,
                        result: stdout.trim()
                    });
                }
            } else {
                resolve({
                    success: false,
                    error: stderr || `Process exited with code ${code}`
                });
            }
        });
        
        child.on('error', (error) => {
            resolve({
                success: false,
                error: `Failed to start interpreter: ${error.message}. Make sure 'susumu' is installed and in your PATH.`
            });
        });
        
        // Send the code to the interpreter
        child.stdin.write(code);
        child.stdin.end();
    });
}

// Show AST in a webview panel
function showASTPanel(context: vscode.ExtensionContext, code: string, ast: string) {
    const panel = vscode.window.createWebviewPanel(
        'susumuAST',
        'Susumu AST',
        vscode.ViewColumn.Two,
        {
            enableScripts: true,
            retainContextWhenHidden: true
        }
    );
    
    panel.webview.html = getASTWebviewContent(code, ast);
}

// Generate webview content for AST display
function getASTWebviewContent(code: string, ast: string): string {
    return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Susumu AST</title>
    <style>
        body {
            font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
            margin: 20px;
            background-color: var(--vscode-editor-background);
            color: var(--vscode-editor-foreground);
        }
        .section {
            margin-bottom: 30px;
        }
        .section h2 {
            color: var(--vscode-textLink-foreground);
            border-bottom: 1px solid var(--vscode-panel-border);
            padding-bottom: 10px;
        }
        .code-block {
            background-color: var(--vscode-textCodeBlock-background);
            border: 1px solid var(--vscode-panel-border);
            padding: 15px;
            border-radius: 5px;
            overflow-x: auto;
            white-space: pre-wrap;
        }
        .ast-tree {
            font-size: 12px;
            line-height: 1.4;
        }
        .arrow-highlight {
            color: #FF6B6B;
            font-weight: bold;
        }
        .function-highlight {
            color: #4ECDC4;
            font-weight: bold;
        }
    </style>
</head>
<body>
    <h1>🌊 Susumu AST Viewer</h1>
    
    <div class="section">
        <h2>Source Code</h2>
        <div class="code-block">${escapeHtml(code)}</div>
    </div>
    
    <div class="section">
        <h2>Abstract Syntax Tree</h2>
        <div class="code-block ast-tree">${escapeHtml(ast)}</div>
    </div>
    
    <div class="section">
        <h2>Flow Visualization</h2>
        <div class="code-block">
            ${generateFlowVisualization(code)}
        </div>
    </div>
</body>
</html>`;
}

function escapeHtml(text: string): string {
    return text
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#039;');
}

function generateFlowVisualization(code: string): string {
    // Simple flow visualization - highlight arrows and functions
    return code
        .replace(/->/g, '<span class="arrow-highlight">→</span>')
        .replace(/<-/g, '<span class="arrow-highlight">←</span>')
        .replace(/\b([a-zA-Z_][a-zA-Z0-9_]*)\s*\(/g, '<span class="function-highlight">$1</span>(');
}