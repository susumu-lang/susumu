# VSCode Extension Testing Guide

## Quick Testing Without Installation Loop

### 1. Pattern Testing Tool
```bash
# Test syntax patterns directly
node test-syntax-patterns.js

# Add new test cases in the testCases array:
{ pattern: 'pattern-name', text: 'code to test' }
```

### 2. Manual Regex Testing
Open browser console and test patterns:
```javascript
// Test a specific regex pattern
const pattern = /\b(success|error|some|none)\s*(<-)\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*(->)\s*\{/;
const text = "success <- receipt -> {";
console.log(text.match(pattern));
```

### 3. Development Extension Host
Instead of installing/uninstalling:
1. Open VSCode in the extension directory
2. Press `F5` to launch Extension Development Host
3. Open a `.susu` file in the dev host
4. Test syntax highlighting in real-time
5. Make changes, reload window (`Ctrl+R`) to see updates

### 4. Grammar Testing Website
Visit: https://macromates.com/manual/en/regular_expressions
Test TextMate patterns online

## Current Test Results
```
📊 Test Results: 15 passed, 1 failed

✅ Multi-line arrow flows: orderData -> validateOrder <- enrichWithDefaults
✅ Match expressions: match { success <- receipt -> { ... } }
✅ Return statements: return <- error <- "Payment failed: " + errorMsg  
✅ String concatenation: "Payment failed: " + errorMsg
✅ Built-in functions: print, validateOrder, sendConfirmation
✅ Variables: receipt, validatedOrder
✅ Functions: validateOrder, calculateTotals
```

## Pattern Categories Fixed

### ✅ Multi-line Arrow Flows
- `orderData ->` (start of chain)
- `    validateOrder <-` (convergent functions) 
- `    enrichWithDefaults <-` (more convergent)
- `    calculateTotals ->` (forward flow)
- `    validatedOrder` (final variable)

### ✅ Match Expressions
- `match {` (match keyword)
- `success <- receipt -> {` (pattern binding with arrows)

### ✅ Complex Return Statements
- `return <- error <- "Payment failed: " + errorMsg` (nested arrows with concatenation)

### ✅ All Function Types
- Built-in functions (print, add, etc.)
- Domain functions (validateOrder, sendConfirmation, etc.)
- User-defined functions in arrow flows

### ✅ Variables
- Property access: `user.name`
- Assignment targets: `testOrder = {}`
- Arrow flow variables: `receipt`, `errorMsg`
- Pattern bindings: `success <- receipt`

## Installation for Testing
```bash
# Install latest version
code --install-extension susumu-language-0.1.4.vsix

# Or use Extension Development Host for faster iteration
```