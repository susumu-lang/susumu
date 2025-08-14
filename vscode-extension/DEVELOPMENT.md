# VS Code Extension Development Guide

This guide explains how to build and test the Susumu VS Code extension locally.

## Prerequisites

1. **Install Node.js** (v16 or later)
2. **Install VS Code** (v1.80.0 or later)
3. **Build the Susumu LSP server**:
   ```bash
   cd ../rust-backend
   cargo build --release --features lsp --bin susumu-lsp
   ```

## Building the Extension

1. **Install dependencies**:
   ```bash
   cd vscode-extension
   npm install
   ```

2. **Compile TypeScript**:
   ```bash
   npm run compile
   ```

3. **Package the extension**:
   ```bash
   npm run package
   # This creates a .vsix file
   ```

## Development Testing

### Method 1: Extension Development Host (Recommended for development)

1. **Open the extension in VS Code**:
   ```bash
   code vscode-extension/
   ```

2. **Press F5** or go to Run > Start Debugging
   - This launches a new "Extension Development Host" window
   - The extension is automatically loaded in this window

3. **Test in the development window**:
   - Create a test file with `.susu` extension
   - Type some Susumu code to test syntax highlighting
   - Test LSP features (if LSP server is installed)

### Method 2: Install from VSIX (Recommended for local use)

1. **Package the extension**:
   ```bash
   npm run package
   ```

2. **Install in your main VS Code**:
   ```bash
   code --install-extension susumu-language-0.1.0.vsix
   ```

3. **Or install via VS Code UI**:
   - Open VS Code
   - Go to Extensions (Ctrl+Shift+X)
   - Click the "..." menu → "Install from VSIX..."
   - Select the `.vsix` file

### Method 3: Symlink Installation (For active development)

```bash
# Create symlink to extension directory
ln -s "$(pwd)/vscode-extension" ~/.vscode/extensions/susumu-language-0.1.0

# Restart VS Code
```

## Testing the Extension

### 1. Create Test Files

Create test files with `.susu` extension:

**test.susu**:
```susumu
// Basic function
square(x) {
    return <- x -> multiply <- x
}

// Arrow chain
5 -> add <- 3 -> multiply <- 2 -> square

// Built-in functions
"Hello" -> concat <- " World" -> to_upper
[1, 2, 3, 4, 5] -> first -> add <- 10
```

### 2. Test Features

#### Syntax Highlighting
- Open a `.susu` file
- Verify colors for:
  - Keywords (`return`, `true`, `false`)
  - Arrow operators (`->`, `<-`)
  - Built-in functions (`add`, `multiply`, etc.)
  - Strings, numbers, comments

#### Language Server Features (requires LSP server)
1. **Auto-completion**: Type `add` and see suggestions
2. **Hover**: Hover over built-in functions for documentation
3. **Diagnostics**: Introduce syntax errors and see red squiggles
4. **Go to Definition**: Right-click function names
5. **Document Symbols**: Open Outline view (Ctrl+Shift+O)

#### Commands
1. **Run File**: Right-click file → "Run Susumu File"
2. **Evaluate Selection**: Select code → Ctrl+Shift+E
3. **Show AST**: Command palette → "Susumu: Show AST"

### 3. LSP Server Setup

For full functionality, ensure the LSP server is available:

```bash
# Build and install LSP server
cd ../rust-backend
cargo install --path . --features lsp --bin susumu-lsp

# Verify it's in PATH
which susumu-lsp
```

If LSP server is not in PATH, configure the path in VS Code settings:
```json
{
  "susumu.lsp.serverPath": "/path/to/susumu-lsp"
}
```

## Configuration

The extension can be configured in VS Code settings:

```json
{
  "susumu.lsp.enabled": true,
  "susumu.lsp.serverPath": "susumu-lsp",
  "susumu.formatting.enabled": true,
  "susumu.diagnostics.enabled": true
}
```

## Debugging

### Extension Development Console

1. In the Extension Development Host window: Ctrl+Shift+I
2. Check Console tab for errors
3. Use `console.log()` in extension code for debugging

### LSP Server Debugging

1. Set environment variable: `RUST_LOG=debug`
2. LSP server logs will appear in VS Code Output panel
3. Select "Susumu Language Server" from dropdown

### Common Issues

**LSP server not starting**:
- Check `susumu-lsp` is in PATH
- Check VS Code Output panel for errors
- Try setting custom path in settings

**Syntax highlighting not working**:
- Ensure file has `.susu` or `.susumu` extension
- Try reloading window (Ctrl+Shift+P → "Reload Window")

**Extension not loading**:
- Check Extensions view for error messages
- Verify extension is enabled
- Check for conflicts with other extensions

## Publishing

### Prepare for publishing:

1. **Update version** in `package.json`
2. **Test thoroughly** on different platforms
3. **Update README.md** with current features
4. **Package**: `npm run package`

### Publish to VS Code Marketplace:

```bash
# Install vsce if not already installed
npm install -g vsce

# Login to marketplace
vsce login your-publisher-name

# Publish
vsce publish
```

## File Structure

```
vscode-extension/
├── package.json          # Extension manifest
├── src/
│   └── extension.ts      # Main extension code
├── syntaxes/
│   └── susumu.tmLanguage.json  # Syntax highlighting
├── language-configuration.json # Language config
├── README.md             # User documentation
└── out/                  # Compiled output
    └── extension.js
```

## Next Steps

1. **Improve LSP server** with better error reporting
2. **Add debugging support** for Susumu code
3. **Implement code formatting** in LSP server
4. **Add snippets** for common patterns
5. **Create test suite** for extension functionality

## Resources

- [VS Code Extension API](https://code.visualstudio.com/api)
- [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
- [TextMate Grammar Guide](https://macromates.com/manual/en/language_grammars)