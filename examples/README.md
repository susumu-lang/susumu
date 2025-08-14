# Susumu Examples

This directory contains example programs showcasing Susumu's arrow-flow programming capabilities, from basic syntax to complex business applications.

## 🚀 Getting Started

### Prerequisites
```bash
# Install Susumu
cargo install susumu

# Or build from source
cd core && cargo build --release
```

### Running Examples
```bash
# Run any example
susumu examples/hello_world.susu

# With debug output
susumu --debug examples/math_pipeline.susu
```

## 📚 Example Programs

### **hello_world.susu**
**Purpose:** Your first Susumu program  
**Features:** Basic arrow syntax, function composition  
**Difficulty:** Beginner  

```susumu
"Hello, Susumu!" -> print
5 -> add <- 10 -> multiply <- 2 -> print  // Output: 30
```

### **math_pipeline.susu**  
**Purpose:** Mathematical function composition  
**Features:** User-defined functions, arrow chaining, step visualization  
**Difficulty:** Beginner  

```susumu
result = 5 -> double -> addTen -> square  // Output: 400
```

### **comprehensive_syntax_test.susu**
**Purpose:** Complete language feature demonstration  
**Features:** All current syntax patterns, data types, control flow  
**Difficulty:** Intermediate  

**Tests:**
- ✅ Numbers, strings, arrays, objects
- ✅ Single-line and multi-line data structures  
- ✅ Arrow operations and function calls
- ✅ Conditional processing
- ✅ Complex nested data

### **ecommerce_order_processing.susu**
**Purpose:** Realistic business application  
**Features:** Complex business logic, error handling, data validation  
**Difficulty:** Advanced  

**Demonstrates:**
- Complete order processing pipeline
- Multi-step validation with error propagation
- Business rule implementation
- Payment processing simulation
- Inventory management workflows

### **arrow_convergence.susu**
**Purpose:** Advanced arrow-flow patterns  
**Features:** Convergent arrows, parallel data processing  
**Difficulty:** Intermediate  

### **pattern_matching.susu** 
**Purpose:** Control flow and pattern matching  
**Features:** Match expressions, conditional branching  
**Difficulty:** Intermediate  

### **visual_debug_demo.susu**
**Purpose:** Visual debugging capabilities  
**Features:** Intentional bugs for debugging demonstration  
**Difficulty:** Advanced  
**Note:** Contains deliberate errors to showcase debugging tools

### **future_vision_showcase.susu** 
**Purpose:** Planned language features  
**Features:** Advanced patterns, type system preview, reactive programming  
**Difficulty:** Advanced  
**Note:** Some features may not yet be implemented

**Planned Features:**
- Object mutations with `<~` operator
- Module system with arrow imports
- Type annotations and safety
- Parallel processing capabilities
- Reactive programming patterns

## 🎯 Usage Recommendations

### **Learning Path:**
1. Start with `hello_world.susu` 
2. Explore `math_pipeline.susu`
3. Study `comprehensive_syntax_test.susu`
4. Analyze `ecommerce_order_processing.susu` 
5. Experiment with `future_vision_showcase.susu`

### **For Different Use Cases:**

**Learning Susumu:**
- `hello_world.susu` - Basic concepts
- `math_pipeline.susu` - Function composition
- `comprehensive_syntax_test.susu` - Complete syntax

**Business Applications:**
- `ecommerce_order_processing.susu` - Complete workflow example
- `pattern_matching.susu` - Decision logic patterns

**Advanced Development:**
- `arrow_convergence.susu` - Complex arrow patterns  
- `visual_debug_demo.susu` - Debugging techniques
- `future_vision_showcase.susu` - Upcoming features

## 🔧 Development

### Testing Examples
```bash
# Test all examples
for file in examples/*.susu; do
    echo "Testing $file..."
    susumu "$file"
done

# Test specific example with debugging
susumu --debug examples/comprehensive_syntax_test.susu
```

### Creating New Examples
1. Follow the existing naming convention
2. Include comprehensive comments
3. Start with a purpose statement
4. Add difficulty level
5. Update this README

## 🚀 Next Steps

- Try modifying the examples to experiment with syntax
- Create your own Susumu programs
- Join the community: [GitHub Discussions](https://github.com/susumu-lang/susumu/discussions)
- Read the full documentation: [docs.susumu.dev](https://docs.susumu.dev)

---

**Happy arrow-flowing! 🏹**