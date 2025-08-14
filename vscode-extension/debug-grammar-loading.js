#!/usr/bin/env node

// Deep grammar debugging - simulate exact VSCode TextMate processing
const fs = require('fs');
const path = require('path');

class VSCodeGrammarDebugger {
    constructor() {
        this.grammarPath = path.join(__dirname, 'syntaxes', 'susumu.tmLanguage.json');
        this.grammar = JSON.parse(fs.readFileSync(this.grammarPath, 'utf8'));
    }

    // Test the exact order VSCode processes patterns
    debugPatternOrdering() {
        console.log('🔍 VSCode Pattern Processing Order\n');
        
        const testLine = 'orderData -> validateOrder <- enrichWithDefaults';
        console.log(`Testing line: "${testLine}"\n`);
        
        // VSCode processes patterns in the order they appear in main "patterns" array
        const mainPatterns = this.grammar.patterns;
        console.log('📋 Main Pattern Processing Order:');
        
        mainPatterns.forEach((pattern, index) => {
            if (pattern.include) {
                const categoryName = pattern.include.replace('#', '');
                console.log(`${index + 1}. ${categoryName}`);
                
                // Check if this category would match our test line
                const category = this.grammar.repository[categoryName];
                if (category && category.patterns) {
                    let matched = false;
                    category.patterns.forEach((subPattern, subIndex) => {
                        if (subPattern.match) {
                            try {
                                const regex = new RegExp(subPattern.match);
                                if (testLine.match(regex)) {
                                    console.log(`   ✅ Pattern ${subIndex}: "${subPattern.match}"`);
                                    console.log(`   📝 Scope: ${subPattern.name}`);
                                    matched = true;
                                }
                            } catch (e) {
                                // Skip invalid regex
                            }
                        }
                    });
                    if (!matched) {
                        console.log(`   ❌ No patterns in ${categoryName} match`);
                    }
                }
            }
        });
    }

    // Check for pattern conflicts that might prevent highlighting  
    checkPatternConflicts() {
        console.log('\n🚨 Pattern Conflict Analysis\n');
        
        const problematicLines = [
            'orderData ->',
            '    validateOrder <-',
            'success <- receipt -> {',
            '"Payment failed: " + errorMsg'
        ];
        
        problematicLines.forEach(line => {
            console.log(`\n📝 Analyzing: "${line}"`);
            const matches = [];
            
            // Check which patterns would match this line
            Object.entries(this.grammar.repository).forEach(([categoryName, category]) => {
                if (category.patterns) {
                    category.patterns.forEach((pattern, index) => {
                        if (pattern.match) {
                            try {
                                const regex = new RegExp(pattern.match);
                                const match = line.match(regex);
                                if (match) {
                                    matches.push({
                                        category: categoryName,
                                        pattern: index,
                                        scope: pattern.name,
                                        matched: match[0],
                                        regex: pattern.match
                                    });
                                }
                            } catch (e) {
                                // Skip invalid regex
                            }
                        }
                    });
                }
            });
            
            if (matches.length === 0) {
                console.log('   ❌ NO PATTERNS MATCH - This explains missing highlighting');
            } else if (matches.length === 1) {
                console.log('   ✅ Single match - should highlight correctly');
                console.log(`   📝 ${matches[0].category}: ${matches[0].scope}`);
            } else {
                console.log(`   ⚠️  Multiple matches (${matches.length}) - potential conflicts:`);
                matches.forEach(match => {
                    console.log(`   - ${match.category}[${match.pattern}]: "${match.matched}" -> ${match.scope}`);
                });
            }
        });
    }

    // Check if our grammar structure matches VSCode expectations
    checkGrammarStructure() {
        console.log('\n🏗️  Grammar Structure Analysis\n');
        
        // Check scopeName format
        if (!this.grammar.scopeName.startsWith('source.')) {
            console.log('❌ scopeName should start with "source." for VSCode');
        } else {
            console.log('✅ scopeName format correct');
        }
        
        // Check file associations in package.json
        const packagePath = path.join(__dirname, 'package.json');
        const pkg = JSON.parse(fs.readFileSync(packagePath, 'utf8'));
        
        const languageConfig = pkg.contributes.languages[0];
        const grammarConfig = pkg.contributes.grammars[0];
        
        if (languageConfig.id !== grammarConfig.language) {
            console.log('❌ Language ID mismatch between language and grammar configs');
        } else {
            console.log('✅ Language configuration matches grammar');
        }
        
        // Check for empty patterns
        let emptyPatterns = 0;
        Object.entries(this.grammar.repository).forEach(([name, category]) => {
            if (!category.patterns || category.patterns.length === 0) {
                console.log(`⚠️  Empty category: ${name}`);
                emptyPatterns++;
            }
        });
        
        if (emptyPatterns === 0) {
            console.log('✅ All pattern categories have content');
        }
    }

    // Generate a minimal test grammar to isolate the issue
    generateMinimalGrammar() {
        console.log('\n🔬 Minimal Grammar Generation\n');
        
        const minimalGrammar = {
            "$schema": "https://raw.githubusercontent.com/martinring/tmlanguage/master/tmlanguage.json",
            "name": "Susumu-Minimal",
            "scopeName": "source.susumu",
            "patterns": [
                {
                    "include": "#test-arrows"
                },
                {
                    "include": "#test-comments"
                }
            ],
            "repository": {
                "test-arrows": {
                    "patterns": [
                        {
                            "match": "->",
                            "name": "keyword.operator.arrow.forward.susumu"
                        },
                        {
                            "match": "<-", 
                            "name": "keyword.operator.arrow.backward.susumu"
                        }
                    ]
                },
                "test-comments": {
                    "patterns": [
                        {
                            "begin": "//",
                            "end": "$",
                            "name": "comment.line.double-slash.susumu"
                        }
                    ]
                }
            }
        };
        
        const minimalPath = path.join(__dirname, 'syntaxes', 'susumu-minimal.tmLanguage.json');
        fs.writeFileSync(minimalPath, JSON.stringify(minimalGrammar, null, 2));
        
        console.log('📁 Created minimal grammar: syntaxes/susumu-minimal.tmLanguage.json');
        console.log('🧪 Test this by temporarily changing package.json grammar path');
        console.log('   If arrows and comments highlight, the issue is pattern complexity');
        console.log('   If they don\'t highlight, the issue is VSCode grammar loading');
    }
}

const grammarDebugger = new VSCodeGrammarDebugger();
grammarDebugger.debugPatternOrdering();
grammarDebugger.checkPatternConflicts(); 
grammarDebugger.checkGrammarStructure();
grammarDebugger.generateMinimalGrammar();