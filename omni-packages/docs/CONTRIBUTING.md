# 🤝 Contributing to Omni Packages Database

Thank you for helping improve package discovery for Omni users worldwide! This guide will help you contribute package metadata, cross-platform mappings, and security information.

## 🎯 **What We Need**

### **1. Package Metadata**
- Cross-platform package mappings (Linux ↔ macOS ↔ Windows)
- Popularity rankings and download statistics
- Security scores and vulnerability information
- Similar/alternative package suggestions

### **2. Platform Coverage**
- **Linux**: apt, dnf, pacman, snap, flatpak, zypper
- **macOS**: brew (formulae and casks)
- **Windows**: winget, chocolatey, scoop

### **3. Categories We're Building**
- Development tools (editors, IDEs, compilers)
- Web browsers and internet tools
- Media players and graphics software
- System utilities and administration tools
- Games and entertainment
- Security and privacy tools

## 📝 **How to Contribute**

### **Step 1: Fork and Clone**
```bash
# Fork the repository on GitHub
git clone https://github.com/YOUR_USERNAME/omni-packages.git
cd omni-packages
```

### **Step 2: Add Package Metadata**

Create a new JSON file in `packages/cross-platform/PACKAGE_NAME.json`:

```json
{
  "name": "package-name",
  "display_name": "Package Display Name",
  "category": "development",
  "description": "Brief description of what this package does",
  "homepage": "https://package-homepage.com",
  "license": "MIT",
  "cross_platform": {
    "linux": {
      "apt": ["package-name"],
      "snap": ["package-name"],
      "flatpak": ["com.example.PackageName"],
      "dnf": ["package-name"],
      "pacman": ["package-name"]
    },
    "macos": {
      "brew": ["package-name"]
    },
    "windows": {
      "winget": ["Publisher.PackageName"],
      "chocolatey": ["package-name"],
      "scoop": ["package-name"]
    }
  },
  "popularity": {
    "rank": 100,
    "downloads_per_month": 1000000,
    "github_stars": 5000,
    "search_frequency": 75
  },
  "security": {
    "score": 8.5,
    "last_audit": "2024-07-05",
    "vulnerabilities": [],
    "cve_count": 0,
    "security_features": [
      "automatic_updates",
      "sandboxing"
    ]
  },
  "similar_packages": [
    "alternative-package-1",
    "alternative-package-2"
  ],
  "alternatives": [
    {
      "name": "alternative-package",
      "reason": "Why this is an alternative"
    }
  ],
  "tags": ["tag1", "tag2", "category"],
  "updated_at": "2024-07-05T00:00:00Z"
}
```

### **Step 3: Validate Your Changes**
```bash
# Install validation dependencies
pip install jsonschema requests pyyaml

# Run validation
python scripts/validate.py
```

### **Step 4: Test API Generation**
```bash
# Generate API endpoints
python scripts/generate-api.py

# Check generated files
ls api/v1/packages/
```

### **Step 5: Submit Pull Request**
```bash
git add .
git commit -m "Add metadata for PACKAGE_NAME"
git push origin main

# Create PR on GitHub
```

## 📊 **Data Sources for Research**

### **Popularity Data**
- **GitHub Stars**: Use GitHub API for open source projects
- **Download Statistics**: 
  - npm: `npm-stat.com`
  - PyPI: `pypistats.org`
  - Homebrew: `formulae.brew.sh`
  - Chocolatey: `chocolatey.org/packages`

### **Security Information**
- **CVE Database**: `cve.mitre.org`
- **Security Advisories**: GitHub Security Advisories
- **Package Scanners**: Snyk, WhiteSource, etc.
- **Update Frequency**: Check package repositories

### **Cross-Platform Research**
- **Official Documentation**: Check package homepages
- **Repository Search**:
  - Linux: `packages.ubuntu.com`, `packages.fedoraproject.org`
  - macOS: `formulae.brew.sh`
  - Windows: `winget.run`, `chocolatey.org`

## ✅ **Quality Guidelines**

### **Package Requirements**
- ✅ Package must be available on at least 2 platforms
- ✅ All package names must be verified and tested
- ✅ Descriptions should be concise and accurate
- ✅ Security scores should be evidence-based
- ✅ Similar packages should be genuinely related

### **Validation Checks**
- JSON syntax validation
- Schema compliance
- Package name consistency
- Cross-platform coverage
- Required field validation

### **Security Scoring Guidelines**
- **9.0-10.0**: Excellent security (regular audits, active maintenance, security features)
- **8.0-8.9**: Good security (regular updates, some security features)
- **7.0-7.9**: Fair security (occasional updates, basic security)
- **6.0-6.9**: Poor security (infrequent updates, security concerns)
- **0.0-5.9**: Avoid (known vulnerabilities, abandoned projects)

## 🎯 **Priority Packages**

Help us add these high-priority packages:

### **Development Tools**
- [ ] Docker / Docker Desktop
- [ ] Node.js / npm
- [ ] Python / pip
- [ ] Java JDK variants
- [ ] Rust / cargo
- [ ] Go compiler

### **Web Browsers**
- [ ] Chrome / Chromium
- [ ] Safari (macOS)
- [ ] Edge
- [ ] Brave
- [ ] Opera

### **Media & Graphics**
- [ ] VLC Media Player
- [ ] GIMP
- [ ] Blender
- [ ] OBS Studio
- [ ] Audacity

### **System Utilities**
- [ ] 7-Zip / Archive utilities
- [ ] File managers
- [ ] System monitors
- [ ] Terminal emulators

## 🚀 **Advanced Contributions**

### **Automation Scripts**
- Package popularity fetchers
- Security vulnerability scanners
- Cross-platform mapping validators
- API endpoint generators

### **Data Quality**
- Update outdated information
- Verify package availability
- Test installation commands
- Report broken packages

## 📞 **Getting Help**

- **Issues**: Report problems or ask questions
- **Discussions**: General discussion about improvements
- **Discord**: Join our community chat (coming soon)
- **Email**: For sensitive security reports

## 🏆 **Contributors**

Contributors will be recognized in:
- Repository README
- API response credits
- Annual contributor highlights
- Optional GitHub sponsor recognition

---

**Together we're building the most comprehensive, community-driven package database for cross-platform package management! 🌟**