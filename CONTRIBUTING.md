# ⚫ Contributing to Omni — Join The Revolution

**Welcome to the movement.** You're not just contributing code—you're helping fix one of computing's most fragmented problems.

**therealcoolnerd** here → and I'm hyped you want to be part of making package management finally make sense across all platforms. This isn't just another open source project—we're building the unified future of software installation. 

```ascii
┌─────────────────────────────────────────────────────────┐
│  ⚪ One codebase, three platforms, infinite possibilities │
└─────────────────────────────────────────────────────────┘
```

## ⚡ Quick Start (Zero Friction)

```bash
# 1. Fork & clone
git clone https://github.com/your-username/omni.git
cd omni

# Keep the official repo handy
git remote add upstream https://github.com/therealcoolnerd/omni.git

# 2. Create your feature branch
git checkout -b feature/game-changing-feature

# 3. Build & test (make sure it works)
cargo build && cargo test

# 4. Commit with style
git commit -m "feat: add feature that changes everything"

# 5. Push & PR
git push origin feature/game-changing-feature
# Then open a Pull Request on GitHub
```

Maintainers preparing a release should ensure `origin` points to the
official repository:

```bash
git remote set-url origin https://github.com/therealcoolnerd/omni.git
```

**That's it.** No complex setup, no bureaucracy—just good code solving real problems.

## 🧠 Development Philosophy (What We Stand For)

- **⚫ Security First**: Never compromise user security for convenience
- **⚪ Universal Compatibility**: Linux, Windows, macOS—same code, same experience
- **⚡ Performance Obsessed**: Sub-200ms operations, minimal memory footprint
- **🎯 UX That Makes Sense**: Complex operations should feel obvious
- **🔬 Test Everything**: Untested code doesn't ship, period

*We're building infrastructure-level software that millions will depend on.*

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

### Git Hooks

No Git hooks are required for this project. Sample hook scripts from `.git/hooks`
are stored in [`docs/git-hook-samples`](docs/git-hook-samples/) if you want to
set up optional checks like running `cargo fmt` or verifying commit messages
before commits.

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
- Email security details privately to: **arealcoolcompany@gmail.com**
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
- **Lead Developer**: @therealcoolnerd
- **Email**: **arealcoolcompany@gmail.com**
- **Twitter**: [@therealcoolnerd](https://twitter.com/therealcoolnerd)

## 🔥 Thank You!

Every contribution, no matter how small, helps fix package management forever. You're not just writing code—you're part of the movement making software installation finally make sense across all platforms.

**Let's build something that actually works.** ⚡

---

<div align="center">

```ascii
┌─────────────────────────────────────────────────────────────┐
│  ⚫ Built with Rust, powered by caffeine, fueled by passion │
└─────────────────────────────────────────────────────────────┘
```

**Built for everyone who's tired of platform fragmentation**

</div>