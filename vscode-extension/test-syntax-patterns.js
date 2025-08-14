#!/usr/bin/env node

// TextMate Grammar Testing Tool
// This allows us to test syntax highlighting patterns without VSCode

const fs = require('fs');
const path = require('path');

// Simple TextMate pattern matcher for testing
class TextMatePatternTester {
    constructor(grammarPath) {
        this.grammar = JSON.parse(fs.readFileSync(grammarPath, 'utf8'));
    }

    // Test a specific pattern against text
    testPattern(patternName, text) {
        console.log(`\n🧪 Testing pattern: ${patternName}`);
        console.log(`📝 Text: "${text}"`);
        
        const patterns = this.grammar.repository[patternName]?.patterns || [];
        let matched = false;
        
        patterns.forEach((pattern, index) => {
            if (pattern.match) {
                const regex = new RegExp(pattern.match);
                const match = text.match(regex);
                if (match) {
                    console.log(`✅ Pattern ${index} matched:`, match[0]);
                    console.log(`   Scope: ${pattern.name}`);
                    if (pattern.captures) {
                        Object.entries(pattern.captures).forEach(([group, capture]) => {
                            if (match[group]) {
                                console.log(`   Group ${group}: "${match[group]}" -> ${capture.name}`);
                            }
                        });
                    }
                    matched = true;
                } else {
                    console.log(`❌ Pattern ${index} no match: ${pattern.match}`);
                }
            } else if (pattern.begin && pattern.end) {
                const beginRegex = new RegExp(pattern.begin);
                const beginMatch = text.match(beginRegex);
                if (beginMatch) {
                    console.log(`✅ Begin pattern ${index} matched:`, beginMatch[0]);
                    console.log(`   Scope: ${pattern.name}`);
                    matched = true;
                } else {
                    console.log(`❌ Begin pattern ${index} no match: ${pattern.begin}`);
                }
            }
        });
        
        if (!matched) {
            console.log(`❌ No patterns matched for "${text}"`);
        }
        
        return matched;
    }

    // Test all problematic cases
    runTests() {
        console.log('🚀 Testing Susumu Syntax Patterns\n');
        
        // Test cases from the user's issues
        const testCases = [
            // Multi-line arrow flows
            { pattern: 'multiline-flows', text: 'orderData ->' },
            { pattern: 'multiline-flows', text: '    validateOrder <-' },
            { pattern: 'multiline-flows', text: '    enrichWithDefaults <-' },
            { pattern: 'multiline-flows', text: '    calculateTotals ->' },
            { pattern: 'multiline-flows', text: '    validatedOrder' },
            
            // Match expressions  
            { pattern: 'match-expressions', text: 'match {' },
            { pattern: 'match-expressions', text: 'success <- receipt -> {' },
            { pattern: 'match-cases', text: 'success <- receipt -> {' },
            { pattern: 'patterns', text: 'success <- receipt' },
            
            // Return statements with string concatenation
            { pattern: 'return-statements', text: 'return <- error <- "Payment failed: " + errorMsg' },
            { pattern: 'strings', text: '"Payment failed: " + errorMsg' },
            
            // Built-in functions
            { pattern: 'builtins', text: 'print' },
            { pattern: 'builtins', text: 'validateOrder' },
            { pattern: 'builtins', text: 'sendConfirmation' },
            
            // Variables and functions
            { pattern: 'variables', text: 'receipt' },
            { pattern: 'functions', text: 'validateOrder' }
        ];
        
        let passed = 0;
        let failed = 0;
        
        testCases.forEach(testCase => {
            if (this.testPattern(testCase.pattern, testCase.text)) {
                passed++;
            } else {
                failed++;
            }
        });
        
        console.log(`\n📊 Test Results: ${passed} passed, ${failed} failed`);
        
        if (failed > 0) {
            console.log('\n🔧 Recommendations:');
            console.log('1. Check regex patterns are properly escaped');
            console.log('2. Verify pattern order (more specific patterns first)');
            console.log('3. Test begin/end patterns for multi-line constructs');
        }
    }
}

// Run the tests
const grammarPath = path.join(__dirname, 'syntaxes', 'susumu.tmLanguage.json');
const tester = new TextMatePatternTester(grammarPath);
tester.runTests();