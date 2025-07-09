#!/bin/bash
# Omni Local Build Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu" 
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-pc-windows-msvc"
)

FEATURES=(
    ""              # No features (minimal build)
    "--features gui"
    "--features ssh"
    "--all-features"
)

BUILD_DIR="target/releases"
PACKAGE_DIR="packages"

print_header() {
    echo -e "${BLUE}============================================${NC}"
    echo -e "${BLUE}  Omni Universal Package Manager Builder  ${NC}"
    echo -e "${BLUE}============================================${NC}"
}

print_step() {
    echo -e "${GREEN}[STEP]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_dependencies() {
    print_step "Checking dependencies..."
    
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo not found. Please install Rust: https://rustup.rs/"
        exit 1
    fi
    
    if ! command -v rustc &> /dev/null; then
        print_error "Rustc not found. Please install Rust: https://rustup.rs/"
        exit 1
    fi
    
    echo "✓ Rust toolchain found"
    echo "  Cargo: $(cargo --version)"
    echo "  Rustc: $(rustc --version)"
}

install_targets() {
    print_step "Installing cross-compilation targets..."
    
    for target in "${TARGETS[@]}"; do
        echo "Installing target: $target"
        rustup target add "$target" || print_warning "Failed to install target $target"
    done
}

install_cross() {
    print_step "Installing cross-compilation tools..."
    
    if ! command -v cross &> /dev/null; then
        echo "Installing cross..."
        cargo install cross
    else
        echo "✓ Cross already installed"
    fi
}

run_tests() {
    print_step "Running tests..."
    
    echo "Running unit tests..."
    cargo test --no-default-features
    
    echo "Running integration tests..."
    cargo test --test integration --no-default-features || print_warning "Integration tests failed"
    
    echo "Running doc tests..."
    cargo test --doc --no-default-features || print_warning "Doc tests failed"
}

lint_code() {
    print_step "Running linting..."
    
    echo "Running rustfmt..."
    cargo fmt --all -- --check || {
        print_warning "Code formatting issues found. Run 'cargo fmt' to fix."
    }
    
    echo "Running clippy..."
    cargo clippy --all-targets --no-default-features -- -D warnings || {
        print_warning "Clippy warnings found."
    }
}

security_audit() {
    print_step "Running security audit..."
    
    if ! command -v cargo-audit &> /dev/null; then
        echo "Installing cargo-audit..."
        cargo install cargo-audit
    fi
    
    cargo audit || print_warning "Security vulnerabilities found"
}

build_for_target() {
    local target=$1
    local features=$2
    local feature_name=${features:-"minimal"}
    
    print_step "Building for $target with $feature_name features..."
    
    local build_cmd="cargo build --release --target $target"
    if [ -n "$features" ]; then
        build_cmd="$build_cmd $features"
    else
        build_cmd="$build_cmd --no-default-features"
    fi
    
    echo "Running: $build_cmd"
    
    if [[ "$target" == *"linux"* ]] && [[ "$target" != "x86_64-unknown-linux-gnu" ]]; then
        # Use cross for non-native Linux targets
        cross build --release --target "$target" ${features:---no-default-features} || {
            print_warning "Cross compilation failed for $target"
            return 1
        }
    else
        eval "$build_cmd" || {
            print_warning "Build failed for $target"
            return 1
        }
    fi
    
    echo "✓ Build completed for $target"
}

build_all() {
    print_step "Building for all targets..."
    
    mkdir -p "$BUILD_DIR"
    
    for target in "${TARGETS[@]}"; do
        # Skip unsupported combinations
        case "$target" in
            *"windows"*)
                if [[ "$OSTYPE" != "msys" && "$OSTYPE" != "cygwin" ]]; then
                    echo "Skipping Windows target on non-Windows host"
                    continue
                fi
                ;;
            *"apple"*)
                if [[ "$OSTYPE" != "darwin"* ]]; then
                    echo "Skipping macOS target on non-macOS host"
                    continue
                fi
                ;;
        esac
        
        # Build minimal version for all targets
        build_for_target "$target" ""
        
        # Copy binary to releases directory
        local binary_name="omni"
        if [[ "$target" == *"windows"* ]]; then
            binary_name="omni.exe"
        fi
        
        local source_path="target/$target/release/$binary_name"
        local dest_path="$BUILD_DIR/omni-$target"
        
        if [ -f "$source_path" ]; then
            cp "$source_path" "$dest_path"
            echo "✓ Binary copied to $dest_path"
        fi
    done
}

create_packages() {
    print_step "Creating distribution packages..."
    
    mkdir -p "$PACKAGE_DIR"
    
    # Create tarball for Linux
    if [ -f "$BUILD_DIR/omni-x86_64-unknown-linux-gnu" ]; then
        echo "Creating Linux tarball..."
        tar -czf "$PACKAGE_DIR/omni-linux-amd64.tar.gz" -C "$BUILD_DIR" omni-x86_64-unknown-linux-gnu
    fi
    
    # Create tarball for macOS
    if [ -f "$BUILD_DIR/omni-x86_64-apple-darwin" ]; then
        echo "Creating macOS tarball..."
        tar -czf "$PACKAGE_DIR/omni-macos-amd64.tar.gz" -C "$BUILD_DIR" omni-x86_64-apple-darwin
    fi
    
    # Create zip for Windows
    if [ -f "$BUILD_DIR/omni-x86_64-pc-windows-msvc" ]; then
        echo "Creating Windows zip..."
        (cd "$BUILD_DIR" && zip "../$PACKAGE_DIR/omni-windows-amd64.zip" omni-x86_64-pc-windows-msvc)
    fi
}

clean() {
    print_step "Cleaning build artifacts..."
    cargo clean
    rm -rf "$BUILD_DIR" "$PACKAGE_DIR"
    echo "✓ Clean completed"
}

show_help() {
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  build     Build for all supported targets"
    echo "  test      Run all tests"
    echo "  lint      Run code linting"
    echo "  audit     Run security audit"
    echo "  package   Create distribution packages"
    echo "  clean     Clean build artifacts"
    echo "  deps      Install build dependencies"
    echo "  all       Run complete build pipeline"
    echo "  help      Show this help message"
    echo ""
    echo "Options:"
    echo "  --target TARGET    Build for specific target only"
    echo "  --release          Build in release mode (default)"
    echo "  --dev              Build in development mode"
    echo ""
    echo "Examples:"
    echo "  $0 build --target x86_64-unknown-linux-gnu"
    echo "  $0 all"
    echo "  $0 test"
}

main() {
    print_header
    
    case "${1:-help}" in
        "build")
            check_dependencies
            build_all
            ;;
        "test")
            check_dependencies
            run_tests
            ;;
        "lint")
            check_dependencies
            lint_code
            ;;
        "audit")
            check_dependencies
            security_audit
            ;;
        "package")
            create_packages
            ;;
        "clean")
            clean
            ;;
        "deps")
            check_dependencies
            install_targets
            install_cross
            ;;
        "all")
            check_dependencies
            install_targets
            install_cross
            run_tests
            lint_code
            security_audit
            build_all
            create_packages
            print_step "Build pipeline completed successfully!"
            ;;
        "help"|"--help"|"-h")
            show_help
            ;;
        *)
            print_error "Unknown command: $1"
            show_help
            exit 1
            ;;
    esac
}

# Handle command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --target)
            TARGET_FILTER="$2"
            shift 2
            ;;
        --dev)
            BUILD_MODE="dev"
            shift
            ;;
        --release)
            BUILD_MODE="release"
            shift
            ;;
        *)
            COMMAND="$1"
            shift
            ;;
    esac
done

main "$COMMAND"