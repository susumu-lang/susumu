# Susumu Language Support for VS Code

A comprehensive Visual Studio Code extension for the Susumu arrow-flow programming language, providing a complete development environment with intelligent code completion, real-time diagnostics, and visual debugging capabilities.

## ✨ Features

### 🎯 Intelligent IntelliSense & Typeahead

**Scope-Aware Completions:**
- ✅ **User-defined functions** with parameter information and line references
- ✅ **Variables in scope** including arrow flow variables and assignments
- ✅ **Function parameters** from current function context
- ✅ **27 built-in functions** (math, string, array, type, I/O operations)
- ✅ **Keywords and conditionals** (`i`, `ei`, `e`, `fe`, `w`, `return`, `error`)
- ✅ **Condition types** (`success`, `valid`, `positive`, `negative`, `zero`, `empty`, `found`)
- ✅ **Annotations** (`@trace`, `@monitor`, `@config`, `@parallel`, `@debug`) with snippet expansion
- ✅ **Arrow operators** (`->`, `<-`) with contextual help

**Smart Prioritization:**
- User-defined functions and parameters appear first
- Built-in functions categorized by functionality
- Variables sorted by relevance and proximity

### 📖 Rich Documentation & Hover

**Context-Aware Hover Information:**
- User-defined function signatures with parameter lists and definition locations
- Variable scope information and declaration context  
- Function parameter details within current function
- Built-in function documentation with usage examples
- Arrow operator explanations with data flow diagrams
- Keyword usage patterns and syntax help

**Signature Help:**
- Real-time parameter hints while typing function calls
- Parameter documentation for both built-in and user-defined functions
- Active parameter highlighting based on cursor position
- Support for both traditional and arrow-flow function syntax

### 🎨 Visual Enhancements & Syntax Highlighting

**Advanced Syntax Highlighting:**
- Arrow operators with special highlighting colors
- Function names, keywords, and condition types
- Annotation syntax (`@trace`, `@monitor`, etc.)
- String literals, numbers, arrays, objects, and comments
- Pattern matching constructs (`some`, `none`)
- Multi-line expression support

**Visual Code Decorations:**
- Arrow operators highlighted in bold red (#FF6B6B)
- Interactive hover tooltips explaining data flow direction
- Status bar integration with Susumu language mode indicator

### Commands
- **Run Susumu File** (`Ctrl+F5`): Execute the current Susumu file
- **Evaluate Selection** (`Ctrl+Shift+E`): Evaluate selected Susumu code
- **Show AST**: Display the Abstract Syntax Tree for debugging

### Smart Features
- **Code Lenses**: Inline "Run Function" and "Show Docs" buttons for functions
- **Arrow Decorations**: Visual emphasis on arrow operators with hover tooltips
- **Status Bar**: Shows Susumu language status when editing .susu files

## Installation

### Prerequisites
1. Install the Susumu language runtime:
   ```bash
   # From the rust-backend directory
   cargo install --path . --features lsp
   ```

2. Make sure `susumu-lsp` is in your PATH

### Install Extension
1. Download the `.vsix` file from releases
2. Install via VS Code: `Extensions > Install from VSIX...`
3. Or install from the marketplace: Search for "Susumu Language Support"

## Configuration

The extension can be configured through VS Code settings:

```json
{
  "susumu.lsp.enabled": true,
  "susumu.lsp.serverPath": "susumu-lsp",
  "susumu.formatting.enabled": true,
  "susumu.diagnostics.enabled": true
}
```

## Usage

### Basic Example
```susumu
// Function definition
square(x) {
    return <- x -> multiply <- x
}

// Arrow-flow computation
5 -> add <- 3 -> multiply <- 2 -> square
```

### Arrow-Flow Syntax
- `->` flows data forward to the next function
- `<-` gathers data from the right into a function
- Multiple `<-` create convergence: `x -> func <- y <- z` means `func(x, y, z)`

### Built-in Functions
The extension provides auto-completion for:
- **Math**: `add`, `subtract`, `multiply`, `divide`, `power`, `sqrt`
- **String**: `concat`, `to_upper`, `to_lower`, `length`, `trim`
- **Array**: `first`, `last`, `sort`, `reverse`, `length`
- **I/O**: `print`, `println`
- **Type**: `type_of`, `to_string`, `to_number`

## Commands

| Command | Shortcut | Description |
|---------|----------|-------------|
| `susumu.runFile` | `Ctrl+F5` | Run the current Susumu file |
| `susumu.evaluateSelection` | `Ctrl+Shift+E` | Evaluate selected code |
| `susumu.showAST` | - | Show Abstract Syntax Tree |

## Troubleshooting

### LSP Server Not Starting
1. Ensure `susumu-lsp` is installed and in PATH
2. Check VS Code output panel for error messages
3. Try setting a custom path in settings: `susumu.lsp.serverPath`

### Syntax Highlighting Not Working
1. Ensure the file has `.susu` or `.susumu` extension
2. Try reloading VS Code window (`Ctrl+Shift+P` > "Reload Window")

### Performance Issues
1. Disable LSP if not needed: `"susumu.lsp.enabled": false`
2. Large files may cause slower responses

## Contributing

1. Fork the repository
2. Make your changes
3. Test with the development setup:
   ```bash
   cd vscode-extension
   npm install
   npm run compile
   # Press F5 to launch Extension Development Host
   ```
4. Submit a pull request

## File Extensions

- `.susu` - Primary Susumu file extension
- `.susumu` - Alternative Susumu file extension

## License

MIT License - see LICENSE file for details.

## Links

- [Susumu Language Documentation](https://susumu-lang.github.io)
- [GitHub Repository](https://github.com/susumu-lang/susumu)
- [Report Issues](https://github.com/susumu-lang/susumu/issues)