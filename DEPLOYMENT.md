# Susumu Deployment Strategy

## Repository Structure

### Current Status (Pre-deployment)
```
susumu/
├── rust-backend/          # Core Rust implementation
├── vscode-extension/      # VSCode language support
├── src/                   # Python prototype (legacy)
├── examples/             # Language examples
└── docs/                 # Documentation
```

### Recommended Production Structure
```
susumu/
├── core/                 # Rust interpreter & LSP (renamed from rust-backend)
├── vscode-extension/     # VSCode extension 
├── docs/                 # Documentation & website
├── examples/            # Language examples
├── scripts/             # Build/deployment automation
├── archive/             # Archived Python prototype
└── .github/             # CI/CD workflows
```

## Deployment Targets

### 1. Cargo/crates.io ⭐ **Primary**
**Target:** `cargo install susumu`
**Status:** Ready for deployment
**Required Steps:**
- [x] Cargo.toml metadata configured
- [x] GitHub Actions workflow ready
- [ ] Crates.io publisher account
- [ ] Initial publish: `cargo publish`

**Benefits:**
- Native Rust ecosystem integration
- Automatic dependency management
- Cross-platform binary distribution

### 2. GitHub Releases ⭐ **Primary**  
**Target:** Pre-built binaries for all platforms
**Status:** Automated via GitHub Actions
**Artifacts:**
- `susumu-linux` (x86_64)
- `susumu-windows.exe` (x86_64)
- `susumu-macos` (x86_64)
- `susumu-*-lsp` (LSP servers)
- `susumu-language-*.vsix` (VSCode extension)

**Release Process:**
1. Commit with message containing `release:`
2. GitHub Actions builds all targets
3. Creates release with binaries attached
4. Version auto-extracted from Cargo.toml

### 3. VSCode Marketplace ⭐ **High Priority**
**Target:** VSCode extension marketplace
**Status:** Extension ready, needs publisher setup
**Required Steps:**
- [ ] Azure DevOps publisher account
- [ ] Marketplace metadata optimization
- [ ] Automated publishing workflow
- [ ] Extension testing on multiple platforms

### 4. Documentation Site 🔧 **Future**
**Target:** `https://susumu-lang.org`
**Technology:** GitHub Pages + mdBook or custom Susumu server
**Features:**
- Interactive code playground (WASM)
- Syntax highlighting for Susumu
- API documentation
- Tutorials and examples

### 5. Python Bindings 🔮 **Long-term**
**Target:** `pip install susumu`
**Technology:** PyO3 + maturin
**Benefits:**
- Python ecosystem integration
- Scientific computing use cases
- Data pipeline tooling

### 6. Node.js Bindings 🔮 **Long-term**
**Target:** `npm install susumu`
**Technology:** napi-rs or WASM
**Benefits:**
- Web development integration
- Frontend tooling
- WASM compilation for browsers

## Release Versioning Strategy

### Semantic Versioning
- **v0.3.0:** Current state (LSP + VSCode extension)
- **v0.4.0:** Module system + imports
- **v0.5.0:** Type inference + static analysis
- **v1.0.0:** Production stability milestone

### Release Branches
- `main` - Stable releases
- `develop` - Development work
- `feature/*` - Feature development
- `release/*` - Release preparation

## Package Metadata

### Cargo.toml
```toml
[package]
name = "susumu"
version = "0.3.0"
description = "Arrow-flow programming language with visual data flow"
license = "MIT"
repository = "https://github.com/susumu-lang/susumu"
documentation = "https://docs.susumu-lang.org"
homepage = "https://susumu-lang.org"
keywords = ["programming-language", "functional", "arrow-flow", "visual"]
categories = ["development-tools", "compilers"]
authors = ["Susumu Contributors"]
edition = "2021"
rust-version = "1.70"

[[bin]]
name = "susumu"
path = "core/src/main.rs"

[[bin]]  
name = "susumu-lsp"
path = "core/src/lsp_main.rs"
required-features = ["lsp"]
```

### VSCode Extension package.json
```json
{
  "name": "susumu-language",
  "displayName": "Susumu Language Support",
  "description": "Arrow-flow programming language support",
  "version": "0.3.0",
  "publisher": "susumu-lang",
  "repository": "https://github.com/susumu-lang/susumu",
  "categories": ["Programming Languages"],
  "keywords": ["susumu", "arrow-flow", "functional"]
}
```

## Marketing & Distribution

### GitHub Repository
- **Organization:** `susumu-lang`
- **Main repo:** `susumu-lang/susumu`
- **Website:** `susumu-lang/susumu-lang.github.io`

### Community Building
- **Discord:** Susumu Language Community
- **Discussions:** GitHub Discussions enabled
- **Contributing:** CONTRIBUTING.md with guidelines
- **Code of Conduct:** Standard contributor covenant

### Documentation Strategy
- **README.md:** Quick start + overview
- **docs/language-reference.md:** Complete language spec
- **examples/:** Progressive tutorials
- **API docs:** Generated from Rust docs
- **Website:** Interactive learning experience

## Security Considerations

### Supply Chain Security
- **Dependency auditing:** `cargo audit` in CI
- **SBOM generation:** Software Bill of Materials
- **Signed releases:** GPG signatures for binaries
- **Vulnerability scanning:** GitHub security advisories

### Code Signing
- **Windows:** Authenticode signing (requires certificate)
- **macOS:** Code signing + notarization
- **Linux:** GPG signatures

## Success Metrics

### Initial Launch (Month 1)
- [ ] 100+ GitHub stars
- [ ] 500+ crates.io downloads
- [ ] 50+ VSCode extension installs
- [ ] 10+ community examples

### Growth Phase (Month 3)
- [ ] 1,000+ GitHub stars
- [ ] 5,000+ crates.io downloads  
- [ ] 500+ VSCode extension installs
- [ ] 5+ external blog posts/articles

### Maturity Phase (Month 6)
- [ ] 5,000+ GitHub stars
- [ ] 25,000+ crates.io downloads
- [ ] 2,000+ VSCode extension installs
- [ ] 10+ third-party packages/libraries

## Deployment Checklist

### Pre-deployment
- [x] Codebase cleanup completed
- [x] Comprehensive testing
- [x] Documentation review
- [x] GitHub Actions workflows
- [ ] Legal review (LICENSE, trademark)

### Deployment Day
- [ ] Cargo publish
- [ ] GitHub release creation
- [ ] VSCode marketplace submission
- [ ] Social media announcement
- [ ] Community notifications

### Post-deployment
- [ ] Monitor GitHub issues
- [ ] Community engagement
- [ ] Performance monitoring
- [ ] User feedback collection
- [ ] Iteration planning

---

**Next Steps:** Repository restructuring + crates.io publisher setup