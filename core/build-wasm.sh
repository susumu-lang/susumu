#!/bin/bash

# Susumu WASM Build Script
# Builds optimized WASM package for production use

set -e

echo "🚀 Building Susumu WASM package..."

# Clean previous build
echo "🧹 Cleaning previous build..."
rm -rf pkg/
rm -rf target/wasm32-unknown-unknown/

# Build for web target with optimizations
echo "🔧 Building WASM (web target)..."
wasm-pack build --target web --features wasm

echo "📝 Note: For Node.js or bundler targets, run:"
echo "   wasm-pack build --target nodejs --features wasm"
echo "   wasm-pack build --target bundler --features wasm"

# Copy demo files to pkg directory
echo "📋 Copying demo files..."
cp demo.html pkg/
cp example-node.js pkg/ 2>/dev/null || echo "   (example-node.js not found, creating...)"
# README is already in pkg/ from wasm-pack

# Create quick start script
cat > pkg/quick-start.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>Susumu WASM Quick Start</title>
</head>
<body>
    <h1>Susumu WASM Quick Start</h1>
    <p>Result: <span id="result"></span></p>
    
    <script type="module">
        import init, { SusumuEngine } from './susumu.js';
        
        async function run() {
            await init();
            const engine = new SusumuEngine();
            const result = engine.execute('5 -> add <- 3 -> multiply <- 2');
            document.getElementById('result').textContent = result.result;
        }
        
        run();
    </script>
</body>
</html>
EOF

# Display package info
echo "📦 Package built successfully!"
echo ""
echo "📁 Generated files:"
echo "   • pkg/          - Web/ES modules target"
echo ""
echo "🌐 To test in browser:"
echo "   • Open http://localhost:8000/pkg/demo.html"
echo "   • Or http://localhost:8000/pkg/quick-start.html"
echo ""
echo "📱 To test in Node.js:"
echo "   • cd pkg && node example-node.js"
echo ""
echo "📚 Package size:"
du -h pkg/susumu_bg.wasm
echo ""
echo "✅ Build complete!"