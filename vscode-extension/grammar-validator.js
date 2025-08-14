#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

// VSCode Grammar Validation Tool
class GrammarValidator {
    constructor() {
        this.grammarPath = path.join(__dirname, 'syntaxes', 'susumu.tmLanguage.json');
        this.packagePath = path.join(__dirname, 'package.json');
    }

    validate() {
        console.log('🔍 VSCode Extension Grammar Validation\n');
        
        // 1. Check grammar file exists and is valid JSON
        if (!fs.existsSync(this.grammarPath)) {
            console.log('❌ Grammar file not found:', this.grammarPath);
            return false;
        }
        
        let grammar;
        try {
            grammar = JSON.parse(fs.readFileSync(this.grammarPath, 'utf8'));
            console.log('✅ Grammar file is valid JSON');
        } catch (e) {
            console.log('❌ Grammar file has invalid JSON:', e.message);
            return false;
        }
        
        // 2. Check required TextMate fields
        const required = ['name', 'scopeName', 'patterns'];
        const missing = required.filter(field => !grammar[field]);
        if (missing.length > 0) {
            console.log('❌ Missing required fields:', missing.join(', '));
            return false;
        }
        console.log('✅ All required TextMate fields present');
        
        // 3. Check package.json grammar registration
        let packageJson;
        try {
            packageJson = JSON.parse(fs.readFileSync(this.packagePath, 'utf8'));
        } catch (e) {
            console.log('❌ Cannot read package.json:', e.message);
            return false;
        }
        
        const grammarContrib = packageJson.contributes?.grammars?.[0];
        if (!grammarContrib) {
            console.log('❌ No grammar contribution found in package.json');
            return false;
        }
        
        if (grammarContrib.scopeName !== grammar.scopeName) {
            console.log('❌ Scope name mismatch:');
            console.log(`   package.json: ${grammarContrib.scopeName}`);
            console.log(`   grammar.json: ${grammar.scopeName}`);
            return false;
        }
        console.log('✅ Grammar registration matches in package.json');
        
        // 4. Check pattern structure
        const totalPatterns = this.countPatterns(grammar.repository || {});
        console.log(`✅ Grammar has ${totalPatterns} patterns across all categories`);
        
        // 5. Test critical patterns
        console.log('\n📋 Critical Pattern Check:');
        const criticalCategories = [
            'comments', 'multiline-flows', 'match-expressions', 
            'match-cases', 'return-statements', 'strings'
        ];
        
        let allCriticalPresent = true;
        criticalCategories.forEach(category => {
            if (grammar.repository[category]) {
                const count = grammar.repository[category].patterns?.length || 0;
                console.log(`   ✅ ${category}: ${count} patterns`);
            } else {
                console.log(`   ❌ ${category}: MISSING`);
                allCriticalPresent = false;
            }
        });
        
        if (!allCriticalPresent) {
            console.log('❌ Critical patterns missing');
            return false;
        }
        
        // 6. Check for common regex issues
        console.log('\n🔧 Regex Pattern Validation:');
        let regexIssues = 0;
        this.validateRegexPatterns(grammar.repository, '', (category, patternIndex, error) => {
            console.log(`   ❌ ${category}[${patternIndex}]: ${error}`);
            regexIssues++;
        });
        
        if (regexIssues === 0) {
            console.log('   ✅ All regex patterns are valid');
        } else {
            console.log(`   ⚠️  Found ${regexIssues} regex issues`);
        }
        
        console.log('\n📊 Grammar Validation Summary:');
        if (regexIssues === 0 && allCriticalPresent) {
            console.log('✅ Grammar file appears to be correctly structured for VSCode');
            console.log('\n💡 If syntax highlighting still doesn\'t work, try:');
            console.log('   1. Use Extension Development Host (F5 in VSCode)');
            console.log('   2. Check VSCode Developer Tools (Help > Toggle Developer Tools)');
            console.log('   3. Verify file extension association');
            console.log('   4. Restart VSCode completely');
            return true;
        } else {
            console.log('❌ Grammar file has issues that need fixing');
            return false;
        }
    }
    
    countPatterns(repository) {
        let count = 0;
        Object.values(repository).forEach(category => {
            if (category.patterns) {
                count += category.patterns.length;
            }
        });
        return count;
    }
    
    validateRegexPatterns(repository, path, onError) {
        Object.entries(repository).forEach(([categoryName, category]) => {
            if (category.patterns) {
                category.patterns.forEach((pattern, index) => {
                    // Test match patterns
                    if (pattern.match) {
                        try {
                            new RegExp(pattern.match);
                        } catch (e) {
                            onError(categoryName, index, `Invalid match regex: ${e.message}`);
                        }
                    }
                    
                    // Test begin/end patterns
                    if (pattern.begin) {
                        try {
                            new RegExp(pattern.begin);
                        } catch (e) {
                            onError(categoryName, index, `Invalid begin regex: ${e.message}`);
                        }
                    }
                    
                    if (pattern.end) {
                        try {
                            new RegExp(pattern.end);
                        } catch (e) {
                            onError(categoryName, index, `Invalid end regex: ${e.message}`);
                        }
                    }
                });
            }
        });
    }
}

// Run validation
const validator = new GrammarValidator();
const isValid = validator.validate();
process.exit(isValid ? 0 : 1);