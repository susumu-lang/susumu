# Contributing to Susumu

Thank you for your interest in contributing to Susumu! This document provides guidelines for contributing to the Susumu arrow-flow programming language.

## 🚀 Getting Started

### Prerequisites

- **Rust** (latest stable) - For the core language implementation
- **Node.js** and **npm** - For the VSCode extension
- **Python** (3.9+) - For development scripts and testing

### Development Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/susumu-lang/susumu.git
   cd susumu
   ```

2. **Build the core language:**
   ```bash
   cd core
   cargo build
   cargo test
   ```

3. **Build the VSCode extension:**
   ```bash
   cd ../vscode-extension
   ./build.sh
   ```

## 🏗️ Project Structure

```
susumu/
├── core/                   # Rust language implementation
│   ├── src/               # Core language sources
│   ├── examples/          # Example Susumu programs
│   └── tests/             # Unit and integration tests
├── vscode-extension/      # VSCode language support
├── docs/                  # Documentation
├── examples/              # Sample Susumu programs
└── archive/               # Archived development files
```

## 🎯 How to Contribute

### 1. **Language Features**

**Parser & Interpreter** (`core/src/`):
- Add new syntax features
- Improve error messages
- Enhance type checking
- Optimize performance

**Examples:**
- New arrow-flow patterns
- Built-in functions
- Control flow constructs

### 2. **Developer Experience**

**VSCode Extension** (`vscode-extension/`):
- Syntax highlighting improvements
- LSP server features
- Code completion
- Debugging support

**Documentation:**
- Language guides
- Tutorial content
- API documentation

### 3. **Testing & Quality**

- Add test cases for new features
- Improve error handling
- Performance benchmarks
- Cross-platform compatibility

## 📝 Contribution Guidelines

### Code Standards

**Rust Code:**
- Follow `rustfmt` formatting
- Use `cargo clippy` for linting
- Write comprehensive tests
- Document public APIs

**TypeScript/JavaScript:**
- Use Prettier for formatting
- Follow ESLint rules
- Type everything explicitly
- Write tests for complex logic

### Commit Messages

Use conventional commit format:
```
type(scope): description

feat(parser): add support for parallel mutations
fix(lsp): resolve colon parsing in objects  
docs(readme): update installation instructions
test(core): add arrow-flow integration tests
```

### Pull Request Process

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/your-feature-name`
3. **Make** your changes following the code standards
4. **Test** thoroughly: `cargo test` and `./build.sh` for extension
5. **Commit** with descriptive messages
6. **Push** to your fork: `git push origin feature/your-feature-name`
7. **Open** a Pull Request with:
   - Clear description of changes
   - Screenshots for UI changes
   - Test results
   - Breaking change notes (if any)

## 🐛 Bug Reports

When reporting bugs, please include:

1. **Susumu version** (`susumu --version`)
2. **Operating system** and version
3. **Steps to reproduce** the issue
4. **Expected behavior**
5. **Actual behavior**
6. **Minimal code example** that demonstrates the bug
7. **Error messages** (full text)

Use the GitHub issue templates when available.

## 💡 Feature Requests

For new features:

1. **Check existing issues** to avoid duplicates
2. **Describe the use case** - what problem does this solve?
3. **Provide examples** - how would the syntax look?
4. **Consider alternatives** - are there other ways to achieve this?
5. **Discuss implementation** - any technical considerations?

## 🎨 Language Design Philosophy

When contributing language features, keep Susumu's core principles in mind:

### **Visual Clarity**
Data flow should be immediately apparent from syntax. Arrows show the journey of data through the system.

### **System-Level Comprehension** 
Entire workloads should be visible and understandable in one place.

### **Functional Core**
Immutability by default, with explicit mutation where needed.

### **Practical Completeness**
Excel at complex workflows while supporting everyday programming needs efficiently.

## 📚 Development Resources

### **Language Implementation:**
- [Rust Book](https://doc.rust-lang.org/book/)
- [Crafting Interpreters](http://craftinginterpreters.com/)
- [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)

### **VSCode Extensions:**
- [VSCode Extension API](https://code.visualstudio.com/api)
- [TextMate Grammar Guide](https://macromates.com/manual/en/language_grammars)

### **Testing:**
- [Rust Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Jest Testing Framework](https://jestjs.io/)

## 🤝 Community

- **Discord**: [Susumu Language Server](https://discord.gg/susumu) *(coming soon)*
- **GitHub Discussions**: For questions and community chat
- **GitHub Issues**: For bugs and feature requests

## 📄 License

By contributing to Susumu, you agree that your contributions will be licensed under the MIT License.

## 🙏 Recognition

Contributors are recognized in:
- Release notes
- CONTRIBUTORS.md file
- Project documentation
- Community showcases

Thank you for making Susumu better! 🚀