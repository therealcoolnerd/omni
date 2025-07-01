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
    ║  🚀 OMNI LITE INSTALLER - The Speed Demon                ║
    ║                                                           ║
    ║  • 865KB binary size                                     ║
    ║  • 18-second build time                                  ║
    ║  • 4 dependencies only                                   ║
    ║  • Zero bloat, maximum speed                             ║
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

detect_package_manager() {
    print_status "Detecting package manager..."
    
    if command -v apt &> /dev/null; then
        echo "apt"
    elif command -v brew &> /dev/null; then
        echo "brew"
    elif command -v winget &> /dev/null; then
        echo "winget"
    else
        echo "none"
    fi
}

install_omni_lite() {
    print_status "Installing Omni Lite..."
    
    # Create temporary directory
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"
    
    # Clone repository
    print_status "Cloning repository..."
    git clone https://github.com/therealcoolnerd/omni.git
    cd omni/omni-lite
    
    # Build Omni Lite
    print_status "Building Omni Lite (this should take ~18 seconds)..."
    START_TIME=$(date +%s)
    
    if cargo build --release; then
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
        cp target/release/omni-lite.exe "$INSTALL_PATH"
        
        # Add to PATH if not already there
        if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
            print_warning "Added $HOME/.local/bin to PATH. Restart your shell or run: source ~/.bashrc"
        fi
    else
        # Linux/macOS
        if [[ $EUID -eq 0 ]]; then
            # Running as root
            cp target/release/omni-lite /usr/local/bin/omni
        else
            # Not root, try sudo
            if sudo cp target/release/omni-lite /usr/local/bin/omni; then
                print_success "Installed to /usr/local/bin/omni"
            else
                # Fallback to user directory
                mkdir -p ~/.local/bin
                cp target/release/omni-lite ~/.local/bin/omni
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
    
    print_success "Omni Lite installed successfully!"
    echo ""
    echo -e "${GREEN}📊 Installation Summary:${NC}"
    echo "• Binary size: $BINARY_SIZE"
    echo "• Build time: ${BUILD_TIME}s"
    echo "• Version: Lite (Speed Demon)"
    echo ""
}

verify_installation() {
    print_status "Verifying installation..."
    
    if command -v omni &> /dev/null; then
        echo ""
        echo -e "${GREEN}🎉 Success! Omni Lite is ready to use!${NC}"
        echo ""
        
        # Show version info
        omni --version 2>/dev/null || echo "Omni Lite installed successfully"
        
        # Detect package manager
        PKG_MGR=$(detect_package_manager)
        if [[ "$PKG_MGR" != "none" ]]; then
            echo "• Detected package manager: $PKG_MGR"
        fi
        
        echo ""
        echo -e "${CYAN}🚀 Try these commands:${NC}"
        echo "  omni install firefox"
        echo "  omni search browser"
        echo "  omni list"
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
    
    echo -e "${BLUE}This script will install Omni Lite - the ultra-minimal universal package manager.${NC}"
    echo ""
    
    # Ask for confirmation
    read -p "Continue with installation? [y/N]: " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Installation cancelled."
        exit 0
    fi
    
    check_dependencies
    install_omni_lite
    verify_installation
    
    echo -e "${GREEN}🎯 Welcome to the universal package management revolution!${NC}"
}

# Run main function
main "$@"