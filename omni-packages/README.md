# 🌟 Omni Packages Database

**Community-driven package metadata for the Omni Universal Package Manager**

This repository contains package metadata, popularity rankings, security scores, and cross-platform mappings to enhance package discovery across Linux, macOS, and Windows.

## 🏗️ **Repository Structure**

```
omni-packages/
├── api/v1/                     # JSON API endpoints
│   ├── packages/
│   │   ├── cross-platform.json
│   │   ├── popular.json
│   │   ├── security.json
│   │   └── categories.json
│   └── platform/
│       ├── linux.json
│       ├── macos.json
│       └── windows.json
├── packages/                   # Individual package metadata
│   ├── cross-platform/        # Cross-platform mappings
│   ├── linux/                 # Linux-specific packages
│   ├── macos/                 # macOS-specific packages
│   └── windows/               # Windows-specific packages
├── scripts/                   # Automation scripts
│   ├── update-data.yml        # GitHub Actions workflow
│   ├── validate.py           # Package data validation
│   └── generate-api.py       # API generation script
└── docs/                     # Documentation
    ├── CONTRIBUTING.md       # Contribution guidelines
    ├── SCHEMA.md            # Package metadata schema
    └── API.md               # API documentation
```

## 📊 **Package Metadata Schema**

Each package entry follows this JSON schema:

```json
{
  "name": "firefox",
  "display_name": "Mozilla Firefox",
  "category": "web-browser",
  "description": "Fast, private & safe web browser",
  "homepage": "https://firefox.com",
  "license": "MPL-2.0",
  "cross_platform": {
    "linux": {
      "apt": ["firefox", "firefox-esr"],
      "snap": ["firefox"],
      "flatpak": ["org.mozilla.firefox"],
      "dnf": ["firefox"],
      "pacman": ["firefox"]
    },
    "macos": {
      "brew": ["firefox", "firefox@esr"]
    },
    "windows": {
      "winget": ["Mozilla.Firefox"],
      "chocolatey": ["firefox"],
      "scoop": ["firefox"]
    }
  },
  "popularity": {
    "rank": 15,
    "downloads_per_month": 50000000,
    "github_stars": 8500
  },
  "security": {
    "score": 8.5,
    "last_audit": "2024-12-01",
    "vulnerabilities": [],
    "cve_count": 0
  },
  "metadata": {
    "install_size_mb": 200,
    "dependencies_count": 45,
    "supported_architectures": ["x86_64", "arm64"],
    "supported_os_versions": {
      "windows": ">=10",
      "macos": ">=10.15",
      "linux": "all"
    }
  },
  "similar_packages": [
    "chromium",
    "brave",
    "opera",
    "vivaldi"
  ],
  "alternatives": [
    {
      "name": "chromium",
      "reason": "Open source alternative"
    },
    {
      "name": "brave",
      "reason": "Privacy-focused browser"
    }
  ],
  "tags": ["browser", "web", "privacy", "open-source"],
  "updated_at": "2024-07-05T00:00:00Z"
}
```

## 🔄 **Automated Updates**

This repository uses GitHub Actions to:
- Validate package metadata on PRs
- Update popularity rankings daily
- Fetch security vulnerability data
- Generate API endpoints
- Deploy to GitHub Pages

## 🤝 **Contributing**

1. Fork this repository
2. Add/update package metadata in the appropriate directory
3. Run validation: `python scripts/validate.py`
4. Submit a Pull Request
5. Community review and merge

## 📡 **API Usage**

The generated API is available at: `https://therealcoolnerd.github.io/omni-packages/api/v1/`

**Examples:**
- All packages: `GET /api/v1/packages/all.json`
- Popular packages: `GET /api/v1/packages/popular.json`
- Cross-platform mappings: `GET /api/v1/packages/cross-platform.json`
- Security data: `GET /api/v1/packages/security.json`
- Linux packages: `GET /api/v1/platform/linux.json`

## 📈 **Statistics**

- **Total Packages**: 2,500+
- **Cross-Platform Mappings**: 800+
- **Security Scores**: 1,200+
- **Contributors**: 50+
- **Updated**: Daily automated updates

---

**This database is community-maintained and free forever. No API keys, no rate limits, no costs.**