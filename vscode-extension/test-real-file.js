#!/usr/bin/env node

// Test against the actual problematic file content
const fs = require('fs');
const path = require('path');

// Real problematic lines from the user's file
const realProblematicLines = [
    // Multi-line arrow flows
    "    orderData -> ",
    "    validateOrder <-",
    "    enrichWithDefaults <-",
    "    calculateTotals ->",
    "    validatedOrder",
    
    // Match expressions
    "    paymentResult -> match {",
    "        success <- receipt -> {",
    "            \"✅ Payment successful!\" -> print",
    "            receipt -> ",
    "            sendConfirmation -> ",
    "            updateInventory -> ",
    "            logSuccess -> ",
    "            return <- success <- \"Payment completed successfully\"",
    "        }",
    
    // Comments
    "// Visual Debugging Demo - Showcases Susumu's debugging capabilities",
    "// Complex payment processing with parallel operations",
    "    // Phase 1: Parallel validation and enrichment",
    
    // String concatenation issues
    "            return <- error <- \"Payment failed: \" + errorMsg",
    "            return <- error <- \"Payment failed for \" + validUser.name + \": \" + paymentError",
    
    // Property access
    "                \"👤 User found: \" + user.name -> print",
    "                notifyUser <- validUser.email ->",
    "                updateUserHistory <- validUser.id ->",
];

class RealFileTester {
    constructor(grammarPath) {
        this.grammar = JSON.parse(fs.readFileSync(grammarPath, 'utf8'));
    }

    testAllPatterns() {
        console.log('🔍 Testing Real Problematic Lines from Susumu File\n');
        
        let passed = 0;
        let failed = 0;
        
        realProblematicLines.forEach((line, index) => {
            console.log(`\n📝 Line ${index + 1}: "${line}"`);
            
            let hasMatch = false;
            
            // Test against all pattern categories
            const categories = [
                'comments', 'functions', 'keywords', 'operators', 'strings', 
                'numbers', 'arrays', 'objects', 'tuples', 'annotations', 
                'patterns', 'builtins', 'variables', 'return-statements', 
                'multiline-flows', 'match-expressions', 'match-cases'
            ];
            
            categories.forEach(category => {
                if (this.grammar.repository[category]) {
                    const patterns = this.grammar.repository[category].patterns || [];
                    patterns.forEach((pattern, patternIndex) => {
                        if (this.testSinglePattern(pattern, line, category, patternIndex)) {
                            hasMatch = true;
                        }
                    });
                }
            });
            
            if (hasMatch) {
                console.log('✅ MATCHED');
                passed++;
            } else {
                console.log('❌ NO MATCH - This line will not be highlighted!');
                failed++;
            }
        });
        
        console.log(`\n📊 Results: ${passed} highlighted, ${failed} NOT highlighted`);
        console.log(`\n❌ These ${failed} lines will appear as plain text in VSCode!`);
        
        return { passed, failed };
    }
    
    testSinglePattern(pattern, text, category, index) {
        try {
            if (pattern.match) {
                const regex = new RegExp(pattern.match);
                const match = text.match(regex);
                if (match) {
                    console.log(`   ✅ ${category}[${index}]: ${pattern.name || 'unnamed'}`);
                    return true;
                }
            } else if (pattern.begin) {
                const beginRegex = new RegExp(pattern.begin);
                const beginMatch = text.match(beginRegex);
                if (beginMatch) {
                    console.log(`   ✅ ${category}[${index}] (begin): ${pattern.name || 'unnamed'}`);
                    return true;
                }
            }
        } catch (e) {
            // Invalid regex, skip
        }
        return false;
    }
}

// Run the test
const grammarPath = path.join(__dirname, 'syntaxes', 'susumu.tmLanguage.json');
const tester = new RealFileTester(grammarPath);
const results = tester.testAllPatterns();

if (results.failed > 0) {
    console.log('\n🔧 ISSUES FOUND - Patterns need fixing!');
    console.log('The lines that show "NO MATCH" will appear as plain text in VSCode.');
}