#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

print_banner() {
    echo -e "${MAGENTA}"
    cat << "EOF"
    ╔═══════════════════════════════════════════════════════════╗
    ║                                                           ║
    ║  🏢 OMNI ENTERPRISE INSTALLER - The Powerhouse           ║
    ║                                                           ║
    ║  • ~50MB binary size                                     ║
    ║  • 120-second build time                                 ║
    ║  • SSH + Transactions + Audit + GUI                     ║
    ║  • Maximum power for infrastructure teams               ║
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
    
    # Check Rust version (Enterprise needs newer features)
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    print_status "Rust version: $RUST_VERSION"
    
    # Check available memory (Enterprise build is memory-intensive)
    if command -v free &> /dev/null; then
        AVAILABLE_MEM=$(free -m | awk 'NR==2{printf "%.0f", $7}')
        if [[ $AVAILABLE_MEM -lt 2048 ]]; then
            print_warning "Low available memory ($AVAILABLE_MEM MB). Enterprise build may be slow."
            print_warning "Consider closing other applications during build."
        fi
    fi
    
    print_success "All dependencies found!"
}

install_omni_enterprise() {
    print_status "Installing Omni Enterprise..."
    
    # Create temporary directory
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"
    
    # Clone repository
    print_status "Cloning repository..."
    git clone https://github.com/therealcoolnerd/omni.git
    cd omni
    
    # Build Omni Enterprise
    print_status "Building Omni Enterprise (this may take up to 2 minutes)..."
    print_status "Building comprehensive feature set - please be patient..."
    START_TIME=$(date +%s)
    
    # Use release profile with enterprise features
    if timeout 300 cargo build --release --features enterprise; then
        END_TIME=$(date +%s)
        BUILD_TIME=$((END_TIME - START_TIME))
        print_success "Build completed in ${BUILD_TIME}s!"
    else
        print_warning "Full build timed out or failed. Trying fallback build..."
        if timeout 300 cargo build --release --features full; then
            END_TIME=$(date +%s)
            BUILD_TIME=$((END_TIME - START_TIME))
            print_success "Fallback build completed in ${BUILD_TIME}s!"
        else
            print_error "Enterprise build failed!"
            echo "Try building Core version instead: curl -sSL https://get-omni.dev/core | sh"
            exit 1
        fi
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
    
    print_success "Omni Enterprise installed successfully!"
    echo ""
    echo -e "${GREEN}📊 Installation Summary:${NC}"
    echo "• Binary size: $BINARY_SIZE"
    echo "• Build time: ${BUILD_TIME}s"
    echo "• Version: Enterprise (Powerhouse)"
    echo "• Features: SSH, Transactions, Audit, GUI, Advanced Security"
    echo ""
}

verify_installation() {
    print_status "Verifying installation..."
    
    if command -v omni &> /dev/null; then
        echo ""
        echo -e "${GREEN}🎉 Success! Omni Enterprise is ready to dominate!${NC}"
        echo ""
        
        # Show version info
        omni --version 2>/dev/null || echo "Omni Enterprise installed successfully"
        
        echo ""
        echo -e "${MAGENTA}🏢 Try these Enterprise features:${NC}"
        echo "  omni info                                    # System overview"
        echo "  omni transaction begin 'test-transaction'   # Atomic operations"
        echo "  omni --ssh server install updates           # Remote management"
        echo "  omni audit scan --quick                      # Security audit"
        echo "  omni gui                                     # GUI interface"
        echo "  omni snapshot create 'enterprise-baseline'  # Advanced snapshots"
        echo ""
        echo -e "${BLUE}📚 Enterprise Documentation: https://github.com/therealcoolnerd/omni/docs${NC}"
        echo -e "${CYAN}📞 Enterprise Support: enterprise@omni.dev${NC}"
        
    else
        print_error "Installation verification failed. Omni command not found in PATH."
        echo "You may need to:"
        echo "1. Restart your shell"
        echo "2. Run: source ~/.bashrc"
        echo "3. Add ~/.local/bin to your PATH manually"
        exit 1
    fi
}

check_enterprise_readiness() {
    print_status "Checking enterprise environment..."
    
    # Check for SSH
    if command -v ssh &> /dev/null; then
        print_success "SSH client found - remote management available"
    else
        print_warning "SSH client not found - install for remote management features"
    fi
    
    # Check for GUI environment
    if [[ -n "$DISPLAY" ]] || [[ "$OSTYPE" == "darwin"* ]] || [[ "$OSTYPE" == "msys" ]]; then
        print_success "GUI environment detected - desktop interface available"
    else
        print_warning "No GUI environment - desktop interface may not work"
    fi
    
    # Check for Docker (for container integration)
    if command -v docker &> /dev/null; then
        print_success "Docker found - container integration available"
    else
        print_warning "Docker not found - install for container management features"
    fi
}

main() {
    print_banner
    
    echo -e "${BLUE}This script will install Omni Enterprise - the most powerful universal package manager.${NC}"
    echo ""
    echo "Enterprise features included:"
    echo "• All Core features (snapshots, manifests, security)"
    echo "• SSH integration for remote server management"
    echo "• Transaction system with atomic operations and rollback"
    echo "• Comprehensive audit trails and compliance reporting"
    echo "• GUI management interface"
    echo "• Container integration (Docker/Podman)"
    echo "• Advanced dependency resolution with AI-powered conflict detection"
    echo ""
    echo -e "${YELLOW}⚠️  Note: Enterprise build requires ~2GB RAM and may take up to 2 minutes${NC}"
    echo ""
    
    # Ask for confirmation
    read -p "Continue with Enterprise installation? [y/N]: " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Installation cancelled."
        echo "Consider Omni Core for balanced features: curl -sSL https://get-omni.dev/core | sh"
        exit 0
    fi
    
    check_dependencies
    check_enterprise_readiness
    install_omni_enterprise
    verify_installation
    
    echo ""
    echo -e "${MAGENTA}🏢 Welcome to enterprise-grade universal package management!${NC}"
    echo ""
    echo "🚀 Next steps for enterprise deployment:"
    echo "1. Configure SSH keys: ssh-keygen -t rsa -b 4096"
    echo "2. Test remote management: omni --ssh testserver info"  
    echo "3. Set up audit logging: omni audit configure"
    echo "4. Create enterprise manifest for your infrastructure"
    echo "5. Launch GUI for visual management: omni gui"
    echo ""
    echo "📞 Enterprise support available at enterprise@omni.dev"
}

# Run main function
main "$@"