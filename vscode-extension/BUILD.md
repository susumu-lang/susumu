# Susumu VSCode Extension - Build Guide

This guide covers building the Susumu VSCode extension with integrated LSP server support.

## Overview

The extension includes:
- **Syntax highlighting** with conflict-free TextMate grammar
- **LSP server** built in Rust with comprehensive language features
- **IntelliSense** with completion, hover, diagnostics, and go-to-definition
- **Snippets** for common Susumu patterns

## Quick Build

### Using Build Script (Recommended)
```bash
# Complete build process
./build.sh
```

This automatically:
1. Builds the Rust LSP server in release mode
2. Copies the binary to the extension
3. Installs Node dependencies
4. Compiles TypeScript
5. Packages the VSIX file

### Using NPM Scripts
```bash
# Build everything step by step
npm run full-build

# Or individual steps:
npm run build-lsp      # Build Rust LSP server
npm run copy-lsp       # Copy LSP binary to extension
npm run compile        # Compile TypeScript
npm run package        # Create VSIX package
```

## Production Deployment

The extension bundles the LSP server binary, making it self-contained:

```
susumu-language-0.2.0.vsix
├── bin/susumu-lsp          # Rust LSP server (1MB)
├── out/extension.js        # Compiled TypeScript
├── syntaxes/               # TextMate grammars
├── snippets/               # Code snippets
└── package.json           # Extension manifest
```

### Cross-Platform Support

For production, build separate extensions for different platforms:

```bash
# Linux (current)
./build.sh

# macOS (requires macOS or cross-compilation)
cd ../rust-backend
cargo build --bin susumu-lsp --features lsp --release --target x86_64-apple-darwin
cp target/x86_64-apple-darwin/release/susumu-lsp ../vscode-extension/bin/susumu-lsp-mac
# Then modify extension.ts to detect platform

# Windows (requires Windows or cross-compilation)
cargo build --bin susumu-lsp --features lsp --release --target x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/susumu-lsp.exe ../vscode-extension/bin/susumu-lsp.exe
```

## Development

### Prerequisites
- **Rust** 1.70+ with `cargo`
- **Node.js** 18+ with `npm`
- **VSCode** with VSCE installed: `npm install -g vsce`

### Development Build
```bash
# Install dependencies
npm install

# Build LSP server for development
npm run build-lsp

# Copy LSP binary
npm run copy-lsp

# Compile and watch TypeScript
npm run watch
```

### Testing

#### Extension Development Host
1. Open this directory in VSCode
2. Press `F5` to launch Extension Development Host
3. Open a `.susu` file to test features

#### Install Locally
```bash
# Install the built extension
code --install-extension susumu-language-0.2.0.vsix

# Uninstall
code --uninstall-extension susumu-lang.susumu-language
```

## LSP Server Features

The bundled Rust LSP server provides:

### ✅ Implemented
- **Diagnostics**: Real-time error detection using proper Susumu parser
- **Completion**: Built-in functions, user functions, keywords, operators
- **Hover**: Documentation for functions and variables
- **Go to Definition**: Jump to function definitions
- **Document Symbols**: Outline view of functions

### 📋 Documentation Examples
- `add` → "Adds two or more numbers together\nExample: 5 -> add <- 3"
- `->` → "Flows data forward to the next function"
- `print` → "Prints a value to the console\nExample: \"Hello\" -> print"

### 🔧 Technical Details
- **Language**: Rust for performance
- **Protocol**: LSP 3.17 compatible
- **Transport**: JSON-RPC over stdio
- **Parser**: Full Susumu AST with proper error handling
- **Memory**: Efficient caching of parsed documents

## File Structure

```
vscode-extension/
├── build.sh                    # Complete build script
├── BUILD.md                   # This file
├── bin/
│   └── susumu-lsp            # Bundled LSP server binary
├── src/
│   └── extension.ts          # Extension entry point with LSP client
├── syntaxes/
│   └── susumu-fixed.tmLanguage.json  # Conflict-free grammar
├── snippets/
│   └── susumu.json           # Code snippets
├── package.json              # Extension manifest + build scripts
└── susumu-language-*.vsix    # Built extension packages
```

## Version History

- **0.2.0**: Integrated Rust LSP server, production build process
- **0.1.6**: Fixed syntax highlighting pattern conflicts  
- **0.1.5**: Minimal grammar testing
- **0.1.0**: Initial extension with basic syntax highlighting

## Troubleshooting

### LSP Server Not Found
```
Error: Susumu LSP server not found at: /path/to/bin/susumu-lsp
```

**Solution**: Run the build process to bundle the LSP binary:
```bash
./build.sh
```

### LSP Server Crashes
Check the Output panel in VSCode (View → Output → Susumu):
```
Starting Susumu Language Server
Susumu LSP initialized
```

If missing, the binary may not have executable permissions:
```bash
chmod +x bin/susumu-lsp
```

### Syntax Highlighting Issues
The grammar is now conflict-free. If issues persist:
1. Reload VSCode window: `Ctrl+Shift+P` → "Developer: Reload Window"
2. Check language mode shows "Susumu" (bottom-right corner)
3. Verify file extension is `.susu` or `.susumu`

## Contributing

1. Make changes to `src/extension.ts` or Rust LSP server
2. Run `./build.sh` to test
3. Use Extension Development Host (`F5`) for debugging
4. Submit PR with test results

---

**Build Status**: ✅ Complete production build process  
**LSP Integration**: ✅ Fully functional Rust LSP server  
**Package Size**: 1.2MB (includes 1MB LSP binary)  
**Platforms**: Linux (current), macOS/Windows (planned)