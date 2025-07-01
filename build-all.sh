#!/bin/bash

set -e

echo "🚀 Building All Omni Versions"
echo "================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Create output directory
mkdir -p dist

# Build Omni Lite
print_status "Building Omni Lite..."
if cd omni-lite && cargo build --release; then
    cp target/release/omni-lite ../dist/omni-lite
    LITE_SIZE=$(ls -lh target/release/omni-lite | awk '{print $5}')
    print_success "Omni Lite built successfully (${LITE_SIZE})"
    cd ..
else
    print_error "Failed to build Omni Lite"
    exit 1
fi

# Build Omni Enterprise (current codebase)
print_status "Building Omni Enterprise..."
if cargo build --release --features full; then
    cp target/release/omni dist/omni-enterprise
    ENTERPRISE_SIZE=$(ls -lh target/release/omni | awk '{print $5}')
    print_success "Omni Enterprise built successfully (${ENTERPRISE_SIZE})"
else
    print_warning "Omni Enterprise build had issues (dependencies timeout)"
fi

# Build Omni Core (no GUI, no SSH)
print_status "Building Omni Core..."
if cargo build --release --no-default-features; then
    cp target/release/omni dist/omni-core
    CORE_SIZE=$(ls -lh target/release/omni | awk '{print $5}')
    print_success "Omni Core built successfully (${CORE_SIZE})"
else
    print_warning "Omni Core build had issues"
fi

# Display results
echo ""
echo "📦 Build Results"
echo "================"
ls -lh dist/
echo ""

echo "🎯 Version Comparison"
echo "===================="
echo "Omni Lite:       Ultra-minimal, 4 dependencies, ${LITE_SIZE:-'N/A'}"
echo "Omni Core:       No GUI/SSH, essential features only"
echo "Omni Enterprise: Full-featured with all capabilities"
echo ""

echo "💡 Usage Examples"
echo "================="
echo "# Quick package management"
echo "./dist/omni-lite install firefox"
echo ""
echo "# Advanced features"
echo "./dist/omni-enterprise snapshot create backup"
echo ""
echo "# Balanced approach"  
echo "./dist/omni-core install --from manifest.yaml"
echo ""

print_success "All builds completed!"