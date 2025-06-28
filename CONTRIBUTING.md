# Contributing to Omni

Welcome to the revolution! 🔥 Omni is changing how Linux package management works, and we need brilliant minds like yours to help build the future.

## 🎯 Quick Start

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Write tests** for your changes
4. **Ensure all tests pass**: `cargo test`
5. **Commit your changes**: `git commit -m 'Add amazing feature'`
6. **Push to your branch**: `git push origin feature/amazing-feature`
7. **Open a Pull Request**

## 🧠 Development Philosophy

- **Security First**: Every change must maintain or improve security
- **Universal Compatibility**: Code must work across all supported Linux distributions
- **Performance Matters**: We're building system-critical software
- **User Experience**: Complex operations should feel simple
- **Test Everything**: If it's not tested, it's not ready

## 🛠️ Development Setup

### Prerequisites
- Rust 1.70+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- SQLite development libraries
- GPG for signature verification
- Linux distribution with package managers for testing

### Environment Setup
```bash
git clone https://github.com/therealcoolnerd/omni.git
cd omni
cargo build
cargo test
```

### Testing Your Changes
```bash
# Unit tests
cargo test

# Integration tests with mock mode
./target/debug/omni --mock install firefox
./target/debug/omni --mock search "text editor"

# Performance benchmarks
cargo bench
```

## 📋 Contribution Areas

### 🐛 Bug Fixes
- Error handling improvements
- Edge case resolution
- Performance optimizations
- Security vulnerability patches

### 📦 New Package Manager Support
We're always looking to support more package managers:
- zypper (openSUSE)
- emerge (Gentoo)
- xbps (Void Linux)
- nix (NixOS)

### 🔒 Security Enhancements
- Additional verification methods
- Trust policy improvements
- Audit trail enhancements
- Privilege escalation prevention

### 🌐 Internationalization
- Multi-language support
- Localized error messages
- Cultural adaptation for different regions

### 📚 Documentation
- User guides and tutorials
- API documentation
- Architecture explanations
- Troubleshooting guides

### 🧪 Testing & Quality
- Test coverage expansion
- Continuous integration improvements
- Performance benchmarking
- Static analysis integration

## 📝 Code Style Guidelines

### Rust Style
- Use `cargo fmt` before committing
- Run `cargo clippy` and fix all warnings
- Follow Rust naming conventions
- Document public APIs with rustdoc

### Git Commit Messages
Follow conventional commit format:
```
type(scope): description

feat(boxes): add zypper support for openSUSE
fix(security): resolve GPG verification edge case
docs(readme): update installation instructions
test(integration): add mock mode test coverage
```

### Code Organization
```
src/
├── main.rs           # CLI interface
├── brain.rs          # Core installation logic
├── resolver.rs       # Dependency resolution
├── security.rs       # Verification systems
├── database.rs       # SQLite operations
├── snapshot.rs       # Rollback functionality
├── search.rs         # Cross-platform search
├── interactive.rs    # User interaction
└── boxes/            # Package manager implementations
    ├── apt.rs
    ├── dnf.rs
    ├── pacman.rs
    └── ...
```

## 🔍 Pull Request Process

### Before Submitting
- [ ] All tests pass locally
- [ ] Code follows style guidelines
- [ ] Changes are documented
- [ ] Security implications considered
- [ ] Performance impact assessed

### PR Description Template
Use our PR template (automatically loaded) and include:
- Clear description of changes
- Motivation and context
- Testing performed
- Screenshots/demos for UI changes
- Breaking changes noted

### Review Process
1. Automated checks must pass (CI/CD, tests, linting)
2. Security review for changes affecting privileges or verification
3. Architecture review for changes affecting core systems
4. Performance review for changes affecting critical paths
5. Final approval from maintainers

## 🚨 Security Contributions

Security is critical for Omni. If you find security issues:

### For Public Security Improvements:
- Open a regular issue/PR with security label
- Include test cases demonstrating the improvement
- Document security implications clearly

### For Vulnerability Reports:
- **DO NOT** open public issues for security vulnerabilities
- Email security details privately to: security@omni-project.org
- Include proof of concept if possible
- Allow 90 days for response before public disclosure

## 🌟 Recognition

Contributors who make significant impacts will be:
- Listed in CONTRIBUTORS.md
- Mentioned in release notes
- Invited to maintainer discussions
- Recognized in project documentation

## 📞 Getting Help

### Community Channels
- **GitHub Discussions**: General questions and feature requests
- **GitHub Issues**: Bug reports and specific problems
- **Matrix Chat**: Real-time community discussions
- **Discord**: Voice chat for complex debugging sessions

### Maintainer Contact
- **Lead Developer**: Josef Douglas Charles McClammey (@therealcoolnerd)
- **Technical Questions**: technical@omni-project.org
- **General Inquiries**: hello@omni-project.org

## 🎉 Thank You!

Every contribution, no matter how small, helps build the future of Linux package management. You're not just writing code—you're part of a revolution that's making Linux more accessible and powerful for everyone.

Let's build something amazing together! 🚀

---
Built with ❤️ for the Linux community