# Susumu Documentation Site Plan

## Vision: Susumu-Powered Documentation

**Ultimate Goal:** Serve the documentation site using a Susumu web framework, demonstrating the language's capabilities for real-world web applications.

**Progression:**
1. **Phase 1:** Static site with syntax highlighting (immediate)
2. **Phase 2:** Interactive playground with WASM (6 months) 
3. **Phase 3:** Susumu web framework + dynamic site (12+ months)

## Phase 1: Static Documentation Site

### Technology Stack
**Primary Option:** mdBook + Custom Syntax Highlighting
```bash
# Structure
docs/
├── book.toml              # mdBook configuration
├── src/
│   ├── SUMMARY.md         # Table of contents
│   ├── introduction.md    # Getting started
│   ├── language-reference/ # Complete language spec
│   ├── examples/          # Tutorials and examples
│   └── api/              # Generated API docs
├── theme/                 # Custom theme with Susumu highlighting
└── static/               # Assets, CSS, JS
```

**Alternative:** Docusaurus or VuePress
- Better interactivity
- React/Vue ecosystem
- More complex setup

### Susumu Syntax Highlighting

**Custom TextMate Grammar for Web:**
```json
{
  "name": "Susumu",
  "scopeName": "source.susumu",
  "patterns": [
    {
      "name": "keyword.arrow.susumu", 
      "match": "->|<-"
    },
    {
      "name": "keyword.control.susumu",
      "match": "\\b(i|e|ei|fe|w|return|error)\\b"
    },
    {
      "name": "entity.name.function.susumu",
      "match": "\\b[a-zA-Z_][a-zA-Z0-9_]*(?=\\s*\\()"
    }
  ]
}
```

**Integration Options:**
1. **Prism.js** - Most compatible with static sites
2. **highlight.js** - Good mdBook integration
3. **Monaco Editor** - VSCode-quality highlighting (for playground)

### Site Structure

```
susumu-lang.org/
├── /                      # Landing page with hero + examples
├── /docs/                 # Main documentation
│   ├── /getting-started   # Installation + hello world
│   ├── /language-guide    # Complete language reference
│   ├── /examples          # Tutorials and patterns
│   ├── /api               # API documentation
│   └── /community         # Contributing, discussions
├── /playground            # Interactive editor (Phase 2)
├── /blog                  # Release notes, tutorials
└── /download              # Binary downloads
```

### Content Strategy

**Landing Page:**
- Hero section with animated arrow flow
- Live code example with syntax highlighting
- Performance metrics and benefits
- Download/installation CTAs

**Getting Started:**
- 5-minute quickstart tutorial
- Installation for all platforms
- "Hello World" to complex example progression
- VSCode extension setup

**Language Guide:**
- Syntax reference with examples
- Control flow patterns
- Function composition techniques
- Error handling strategies
- Best practices and patterns

**Examples:**
- Progressive tutorials (beginner → advanced)
- Real-world use cases (data processing, business logic)
- Comparison with other languages
- Performance benchmarks

### Technical Implementation

**mdBook Setup:**
```toml
# book.toml
[book]
title = "Susumu Programming Language"
authors = ["Susumu Contributors"]
language = "en"
multilingual = false
src = "src"

[build]
build-dir = "book"

[preprocessor.links]

[output.html]
additional-css = ["theme/susumu.css"]
additional-js = ["theme/susumu-highlight.js"]
git-repository-url = "https://github.com/susumu-lang/susumu"
edit-url-template = "https://github.com/susumu-lang/susumu/edit/main/docs/{path}"

[output.html.playground]
editable = true
copyable = true
line-numbers = true
```

**Custom CSS for Arrow Highlighting:**
```css
/* theme/susumu.css */
.arrow-forward { color: #4a9eff; font-weight: bold; }
.arrow-backward { color: #ff6b4a; font-weight: bold; }
.susumu-function { color: #50c878; }
.susumu-keyword { color: #ff69b4; font-weight: bold; }

/* Animated arrow flows */
@keyframes flow {
  0% { opacity: 0.5; transform: translateX(-5px); }
  100% { opacity: 1; transform: translateX(0); }
}

.arrow-forward::before {
  content: "→";
  animation: flow 0.5s ease-in-out;
}
```

## Phase 2: Interactive Playground

### WASM Compilation
```rust
// lib.rs - WASM bindings
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SusumuInterpreter {
    // ... implementation
}

#[wasm_bindgen]
impl SusumuInterpreter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> SusumuInterpreter { ... }
    
    #[wasm_bindgen]
    pub fn execute(&mut self, code: &str) -> String { ... }
    
    #[wasm_bindgen]
    pub fn get_ast(&self, code: &str) -> String { ... }
}
```

### Playground Features
- **Live editing** with Monaco Editor
- **Real-time execution** via WASM
- **AST visualization** showing parsed structure
- **Error highlighting** with helpful messages
- **Share links** for code examples
- **Performance metrics** display

### Monaco Editor Integration
```javascript
// Susumu language definition for Monaco
monaco.languages.register({ id: 'susumu' });
monaco.languages.setMonarchTokensProvider('susumu', {
  tokenizer: {
    root: [
      [/->/, 'arrow.forward'],
      [/<-/, 'arrow.backward'],
      [/\b(i|e|ei|fe|w|return|error)\b/, 'keyword'],
      [/[a-zA-Z_]\w*(?=\s*\()/, 'function'],
    ]
  }
});
```

## Phase 3: Susumu Web Framework

### Vision: Laravel-style Framework for Susumu

**Framework Structure:**
```susumu
// routes.susu - Route definitions
routes() {
    "/" -> home_controller
    "/docs" -> docs_controller <- cache_middleware
    "/api/execute" -> api_controller <- auth_middleware
}

// controllers/home.susu
home_controller(request) {
    request -> 
    load_examples <- 
    render_template <- "home.html" ->
    return <- response
}

// middleware/auth.susu  
auth_middleware(request) {
    request -> validate_token -> i authenticated {
        request -> return <- request
    } e {
        request -> error <- unauthorized_response
    }
}
```

**Framework Features:**
- **Routing:** Arrow-based route definitions
- **Middleware:** Pipeline-style request processing
- **Templates:** Susumu-powered template engine
- **ORM:** Database operations with arrow flows
- **Authentication:** Sanctum-style API tokens
- **Queues:** Background job processing
- **Cache:** Redis/Memcached integration
- **Email:** SMTP + template support

### Frontend Framework: Elmish for Susumu

**Component Definition:**
```susumu
// components/counter.susu
counter_component(props) {
    state = {count: 0}
    
    increment() {
        state -> update <- {count: state.count + 1}
    }
    
    render() {
        state -> template <- "
            <div class='counter'>
                <button onclick='{{increment}}'>{{state.count}}</button>
            </div>
        "
    }
}

// Compiles to vanilla JS/HTML/CSS
```

**Benefits:**
- **Type safety:** Compile-time verification
- **Arrow-based state management:** Visual data flow
- **No runtime:** Compiles to optimized vanilla code
- **Susumu consistency:** Same language everywhere

## Implementation Timeline

### Immediate (Week 1)
- [ ] Set up mdBook documentation site
- [ ] Create custom Susumu syntax highlighting
- [ ] Write core documentation content
- [ ] Deploy to GitHub Pages

### Short-term (Month 1)
- [ ] Complete language reference documentation
- [ ] Add interactive examples (static)
- [ ] Implement search functionality
- [ ] SEO optimization

### Medium-term (Month 3-6)
- [ ] WASM compilation working
- [ ] Interactive playground deployed
- [ ] Advanced syntax highlighting (semantic)
- [ ] Mobile-responsive design

### Long-term (Month 6-12)
- [ ] Web framework MVP
- [ ] Frontend framework prototype
- [ ] Self-hosted documentation site
- [ ] Performance optimization

## Success Metrics

### Documentation Quality
- [ ] 100% language feature coverage
- [ ] Progressive tutorial sequence
- [ ] Searchable content
- [ ] Mobile accessibility

### User Engagement  
- [ ] 1000+ monthly unique visitors
- [ ] 10+ minute average session duration
- [ ] <2% bounce rate on tutorials
- [ ] 50+ community examples submitted

### Technical Performance
- [ ] <2s page load times
- [ ] 95+ Lighthouse scores
- [ ] WASM playground <500ms startup
- [ ] Cross-browser compatibility

---

**Next Priority:** Set up basic mdBook site with custom Susumu syntax highlighting