<div align="center">
  <img src="assets/logo.svg" alt="Susumu Logo" width="200" height="200">
  
  # Susumu - Arrow-Flow Programming Language

  [![Rust](https://github.com/susumu-lang/susumu/workflows/CI/badge.svg)](https://github.com/susumu-lang/susumu/actions)
  [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
  [![Crates.io](https://img.shields.io/crates/v/susumu.svg)](https://crates.io/crates/susumu)
</div>

Susumu is a revolutionary functional programming language that uses arrow syntax to make data flow visually explicit. Instead of forcing developers to mentally trace data through complex systems, Susumu makes the data journey visible through arrows.

## 🚀 Quick Start

### Installation

**From Cargo (Rust):**
```bash
cargo install susumu
```

**From GitHub Releases:**
```bash
# Download binary for your platform from GitHub releases
curl -L https://github.com/susumu-lang/susumu/releases/latest/download/susumu-linux -o susumu
chmod +x susumu
```

**VSCode Extension:**
Install the "Susumu Language Support" extension from the VSCode marketplace for syntax highlighting, LSP support, and integrated development features.

### Hello World

Create a file `hello.susu`:
```susumu
main() {
    "Hello, Susumu!" -> print
}
```

Run it:
```bash
susumu hello.susu
```

## 📖 Language Overview

### Core Concept: Visual Data Flow

Traditional code hides data flow:
```python
# Python - data flow is hidden
result = process_data(validate_input(user_input))
```

Susumu makes data flow explicit:
```susumu
# Susumu - data flow is visual
user_input -> validate_input -> process_data -> result
```

### Basic Syntax

**Arrow Operations:**
```susumu
5 -> double -> add_ten -> print    # Forward flow: 5 → double → add_ten → print
data -> process <- config          # Convergence: data and config flow into process
```

**Functions:**
```susumu
double(x) { x -> multiply <- 2 }
process_order(order) {
    order -> validate -> calculate_total -> charge_payment
}
```

**Control Flow:**
```susumu
user -> authenticate -> i success {
    user -> load_dashboard -> return <- dashboard
} e {
    user -> redirect_login -> error <- "Authentication failed"
}
```

**Complex Example:**
```susumu
process_payment(order, payment_method) {
    order -> validate_items -> i valid {
        (order, payment_method) -> charge_card -> i success {
            order -> update_inventory -> 
            send_confirmation -> 
            return <- order_confirmation
        } e {
            order -> refund -> 
            notify_failure -> 
            error <- payment_error
        }
    } e {
        order -> error <- validation_error
    }
}
```

## 🔧 Development Setup

### Prerequisites
- Rust 1.70+ 
- Node.js 18+ (for VSCode extension development)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/susumu-lang/susumu.git
cd susumu

# Build the interpreter
cargo build --release

# Build the LSP server
cargo build --bin susumu-lsp --features lsp --release

# Run tests
cargo test
```

### VSCode Extension Development

```bash
cd vscode-extension
npm install
npm run compile
npm run package  # Creates susumu-language-x.x.x.vsix
```

## 📚 Documentation

- **Language Reference:** [docs/language-reference.md](docs/language-reference.md)
- **Examples:** [examples/](examples/)
- **VSCode Extension:** [vscode-extension/README.md](vscode-extension/README.md)
- **API Documentation:** [docs.rs/susumu](https://docs.rs/susumu)

## 🎯 Key Features

### Visual Debugging Revolution
Traditional debugging requires mental tracing through code. Susumu's arrow syntax makes data flow immediately visible:

```susumu
// Bug immediately visible: payment processed before validation
order -> charge_payment -> validate_order  // ❌ Wrong order!

// Correct flow is obvious:
order -> validate_order -> charge_payment  // ✅ Correct
```

### Production Benefits
- **10x faster debugging:** Visual flow eliminates guesswork
- **Self-documenting code:** Arrows ARE the documentation  
- **Instant onboarding:** New developers understand flow at a glance
- **System-level comprehension:** Entire pipelines visible in one file

### Performance
- **Sub-millisecond parsing:** <1ms for typical programs
- **Lean binaries:** 972KB interpreter, 2.6MB LSP server
- **Fast compilation:** 13.6s clean release build

## 🛠️ Architecture

### Repository Structure
```
susumu/
├── core/                  # Core Rust interpreter
│   ├── src/
│   │   ├── lexer.rs      # Tokenization
│   │   ├── parser.rs     # AST generation  
│   │   ├── interpreter.rs # Execution engine
│   │   ├── lsp.rs        # Language Server Protocol
│   │   ├── ast.rs        # Abstract Syntax Tree
│   │   ├── builtins.rs   # Built-in functions
│   │   └── main.rs       # CLI entry point
│   ├── Cargo.toml        # Rust package manifest
│   └── tests/            # Integration tests
├── vscode-extension/      # VSCode language support
│   ├── src/              # Extension source
│   ├── syntaxes/         # TextMate grammars
│   └── package.json      # Extension manifest
├── examples/             # Example programs (.susu files)
├── docs/                 # Documentation
├── assets/               # Project assets (logo, etc.)
└── scripts/              # Development and testing scripts
```

### Deployment Targets
- **Core Language:** Cargo package on crates.io
- **VSCode Extension:** VSCode Marketplace
- **Binaries:** GitHub Releases (Linux, macOS, Windows)

## 🚧 Development Status

### ✅ Completed (v0.3.0)
- Core language interpreter with arrow syntax
- Full LSP server with diagnostics, completion, hover
- VSCode extension with syntax highlighting
- Complex control flow and error handling
- Production-ready build pipeline

### 🔄 In Progress
- Assignment parsing edge cases
- Module system design
- Standard library expansion

### 📋 Roadmap
- **v0.4.0:** Module system and imports
- **v0.5.0:** Type inference and static analysis  
- **v1.0.0:** Production stability and ecosystem
- **Future:** Web framework, frontend compiler, documentation site

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Quick Contribution Steps
1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes and add tests
4. Run the test suite: `cargo test`
5. Submit a pull request

### Development Priorities
1. **Language Core:** Parser improvements, type system
2. **Developer Experience:** IDE support, debugging tools
3. **Ecosystem:** Standard library, package management
4. **Performance:** Optimization, compilation targets

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🌟 Vision

**The core insight:** Traditional debugging is detective work. Susumu debugging is watching a movie of your data's journey.

Susumu transforms programming from hidden complexity to visual clarity. Our goal is to make complex data flows as easy to understand as following arrows on a map.

---

**Built with ❤️ by the Susumu community**

[Website](https://susumu.dev) • [Documentation](https://docs.susumu.dev) • [Contributing](CONTRIBUTING.md)