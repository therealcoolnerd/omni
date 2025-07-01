#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

print_banner() {
    echo -e "${CYAN}"
    cat << "EOF"
    ╔═══════════════════════════════════════════════════════════╗
    ║                                                           ║
    ║  ⚖️ OMNI CORE INSTALLER - The Sweet Spot                 ║
    ║                                                           ║
    ║  • ~10MB binary size                                     ║
    ║  • 45-second build time                                  ║
    ║  • Snapshots + Manifests                                 ║
    ║  • Perfect balance of features and speed                 ║
    ║                                                           ║
    ╚═══════════════════════════════════════════════════════════╝
EOF
    echo -e "${NC}"
}

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

check_dependencies() {
    print_status "Checking dependencies..."
    
    # Check for Rust
    if ! command -v cargo &> /dev/null; then
        print_error "Rust is required but not installed."
        echo "Install Rust from: https://rustup.rs/"
        echo "Or run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    # Check for Git
    if ! command -v git &> /dev/null; then
        print_error "Git is required but not installed."
        exit 1
    fi
    
    print_success "All dependencies found!"
}

install_omni_core() {
    print_status "Installing Omni Core..."
    
    # Create temporary directory
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"
    
    # Clone repository
    print_status "Cloning repository..."
    git clone https://github.com/therealcoolnerd/omni.git
    cd omni
    
    # Build Omni Core
    print_status "Building Omni Core (this should take ~45 seconds)..."
    START_TIME=$(date +%s)
    
    if cargo build --release --no-default-features --features core; then
        END_TIME=$(date +%s)
        BUILD_TIME=$((END_TIME - START_TIME))
        print_success "Build completed in ${BUILD_TIME}s!"
    else
        print_error "Build failed!"
        exit 1
    fi
    
    # Install binary
    print_status "Installing binary..."
    
    # Detect installation method
    if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
        # Windows
        INSTALL_PATH="$HOME/.local/bin/omni.exe"
        mkdir -p "$HOME/.local/bin"
        cp target/release/omni.exe "$INSTALL_PATH"
    else
        # Linux/macOS
        if [[ $EUID -eq 0 ]]; then
            # Running as root
            cp target/release/omni /usr/local/bin/omni
        else
            # Not root, try sudo
            if sudo cp target/release/omni /usr/local/bin/omni; then
                print_success "Installed to /usr/local/bin/omni"
            else
                # Fallback to user directory
                mkdir -p ~/.local/bin
                cp target/release/omni ~/.local/bin/omni
                print_warning "Installed to ~/.local/bin/omni (add to PATH if needed)"
                
                # Add to PATH if not already there
                if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
                    echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
                    print_warning "Added ~/.local/bin to PATH. Restart your shell or run: source ~/.bashrc"
                fi
            fi
        fi
    fi
    
    # Cleanup
    cd /
    rm -rf "$TEMP_DIR"
    
    # Get binary size
    BINARY_SIZE=$(du -h "$(which omni 2>/dev/null || echo ~/.local/bin/omni)" 2>/dev/null | cut -f1 || echo "N/A")
    
    print_success "Omni Core installed successfully!"
    echo ""
    echo -e "${GREEN}📊 Installation Summary:${NC}"
    echo "• Binary size: $BINARY_SIZE"
    echo "• Build time: ${BUILD_TIME}s"
    echo "• Version: Core (Sweet Spot)"
    echo "• Features: Snapshots, Manifests, Enhanced Security"
    echo ""
}

verify_installation() {
    print_status "Verifying installation..."
    
    if command -v omni &> /dev/null; then
        echo ""
        echo -e "${GREEN}🎉 Success! Omni Core is ready to use!${NC}"
        echo ""
        
        # Show version info
        omni --version 2>/dev/null || echo "Omni Core installed successfully"
        
        echo ""
        echo -e "${CYAN}⚖️ Try these Core features:${NC}"
        echo "  omni install git nodejs docker"
        echo "  omni snapshot create 'clean-state'"
        echo "  omni manifest install team-setup.yaml"
        echo "  omni search browser"
        echo "  omni info"
        echo ""
        echo -e "${BLUE}📚 Documentation: https://github.com/therealcoolnerd/omni${NC}"
        
    else
        print_error "Installation verification failed. Omni command not found in PATH."
        echo "You may need to:"
        echo "1. Restart your shell"
        echo "2. Run: source ~/.bashrc"
        echo "3. Add ~/.local/bin to your PATH manually"
        exit 1
    fi
}

main() {
    print_banner
    
    echo -e "${BLUE}This script will install Omni Core - the balanced universal package manager.${NC}"
    echo ""
    echo "Features included:"
    echo "• All Lite features (universal package management)"
    echo "• Snapshot system for safe experimentation"
    echo "• Manifest support for team coordination"
    echo "• Enhanced security verification"
    echo "• Advanced configuration options"
    echo ""
    
    # Ask for confirmation
    read -p "Continue with installation? [y/N]: " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Installation cancelled."
        exit 0
    fi
    
    check_dependencies
    install_omni_core
    verify_installation
    
    echo -e "${GREEN}⚖️ Welcome to balanced universal package management!${NC}"
    echo "Next steps:"
    echo "1. Create your first snapshot: omni snapshot create 'initial'"
    echo "2. Try a manifest: echo 'packages: [git, nodejs]' > test.yaml && omni manifest install test.yaml"
    echo "3. Explore: omni --help"
}

# Run main function
main "$@"