#!/bin/bash

# Susumu VSCode Extension Build Script
# Builds LSP server, bundles it, and packages the extension

set -e  # Exit on error

echo "🚀 Building Susumu VSCode Extension with LSP Server"
echo "=================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
RUST_BACKEND_DIR="$SCRIPT_DIR/../core"
EXTENSION_DIR="$SCRIPT_DIR"

echo -e "${YELLOW}Step 1: Building Rust LSP Server${NC}"
cd "$RUST_BACKEND_DIR"

if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Cargo.toml not found in core directory${NC}"
    exit 1
fi

# Build LSP server in release mode
echo "Building susumu-lsp binary..."
cargo build --bin susumu-lsp --features lsp --release

if [ ! -f "target/release/susumu-lsp" ]; then
    echo -e "${RED}Error: LSP server binary not created${NC}"
    exit 1
fi

echo -e "${GREEN}✓ LSP server built successfully${NC}"

echo -e "${YELLOW}Step 2: Preparing Extension Directory${NC}"
cd "$EXTENSION_DIR"

# Create bin directory if it doesn't exist
mkdir -p bin

# Copy LSP binary to extension
echo "Copying LSP binary to extension..."
cp "$RUST_BACKEND_DIR/target/release/susumu-lsp" "bin/"

# Make it executable
chmod +x "bin/susumu-lsp"

# Verify binary works
echo "Testing LSP binary..."
if ./bin/susumu-lsp --help >/dev/null 2>&1 || echo '{"jsonrpc":"2.0","id":0,"method":"exit"}' | timeout 2 ./bin/susumu-lsp >/dev/null 2>&1; then
    echo -e "${GREEN}✓ LSP binary is functional${NC}"
else
    echo -e "${RED}Warning: LSP binary test failed, but continuing...${NC}"
fi

echo -e "${YELLOW}Step 3: Installing Node Dependencies${NC}"
if [ ! -f "package.json" ]; then
    echo -e "${RED}Error: package.json not found${NC}"
    exit 1
fi

npm install

echo -e "${YELLOW}Step 4: Compiling TypeScript${NC}"
npm run compile

echo -e "${YELLOW}Step 5: Packaging Extension${NC}"
npm run package

# Find the created VSIX file
VSIX_FILE=$(find . -name "*.vsix" -type f -printf '%T@ %p\n' | sort -n | tail -1 | cut -d' ' -f2-)

if [ -f "$VSIX_FILE" ]; then
    echo -e "${GREEN}✓ Extension packaged successfully!${NC}"
    echo -e "${GREEN}📦 Package: $VSIX_FILE${NC}"
    
    # Show package info
    PACKAGE_SIZE=$(du -h "$VSIX_FILE" | cut -f1)
    echo -e "${GREEN}📊 Size: $PACKAGE_SIZE${NC}"
    
    # Verify LSP is included
    if unzip -l "$VSIX_FILE" | grep -q "bin/susumu-lsp"; then
        echo -e "${GREEN}✓ LSP server binary included in package${NC}"
    else
        echo -e "${RED}⚠️  Warning: LSP binary not found in package${NC}"
    fi
    
    echo ""
    echo "🎉 Build Complete!"
    echo "To install: code --install-extension $VSIX_FILE"
    echo "To test: Open a .susu file in VSCode"
    
else
    echo -e "${RED}Error: VSIX package not created${NC}"
    exit 1
fi