#!/usr/bin/env python3
"""
Package metadata validation script for Omni Packages Database
"""

import json
import os
import sys
from pathlib import Path
from typing import Dict, List, Any
import jsonschema

# Package metadata JSON schema
PACKAGE_SCHEMA = {
    "type": "object",
    "required": ["name", "display_name", "category", "description", "cross_platform"],
    "properties": {
        "name": {"type": "string", "pattern": "^[a-z0-9-_]+$"},
        "display_name": {"type": "string"},
        "category": {"type": "string"},
        "description": {"type": "string"},
        "homepage": {"type": "string", "format": "uri"},
        "license": {"type": "string"},
        "cross_platform": {
            "type": "object",
            "properties": {
                "linux": {
                    "type": "object",
                    "properties": {
                        "apt": {"type": "array", "items": {"type": "string"}},
                        "snap": {"type": "array", "items": {"type": "string"}},
                        "flatpak": {"type": "array", "items": {"type": "string"}},
                        "dnf": {"type": "array", "items": {"type": "string"}},
                        "pacman": {"type": "array", "items": {"type": "string"}},
                        "zypper": {"type": "array", "items": {"type": "string"}}
                    }
                },
                "macos": {
                    "type": "object",
                    "properties": {
                        "brew": {"type": "array", "items": {"type": "string"}}
                    }
                },
                "windows": {
                    "type": "object", 
                    "properties": {
                        "winget": {"type": "array", "items": {"type": "string"}},
                        "chocolatey": {"type": "array", "items": {"type": "string"}},
                        "scoop": {"type": "array", "items": {"type": "string"}}
                    }
                }
            }
        },
        "popularity": {
            "type": "object",
            "properties": {
                "rank": {"type": "integer", "minimum": 1},
                "downloads_per_month": {"type": "integer", "minimum": 0},
                "github_stars": {"type": "integer", "minimum": 0},
                "search_frequency": {"type": "integer", "minimum": 0, "maximum": 100}
            }
        },
        "security": {
            "type": "object",
            "properties": {
                "score": {"type": "number", "minimum": 0, "maximum": 10},
                "last_audit": {"type": "string", "format": "date"},
                "vulnerabilities": {"type": "array"},
                "cve_count": {"type": "integer", "minimum": 0},
                "security_features": {"type": "array", "items": {"type": "string"}}
            }
        },
        "similar_packages": {"type": "array", "items": {"type": "string"}},
        "alternatives": {
            "type": "array",
            "items": {
                "type": "object",
                "required": ["name", "reason"],
                "properties": {
                    "name": {"type": "string"},
                    "reason": {"type": "string"}
                }
            }
        },
        "tags": {"type": "array", "items": {"type": "string"}},
        "updated_at": {"type": "string", "format": "date-time"}
    }
}

def validate_package(file_path: Path) -> List[str]:
    """Validate a single package file"""
    errors = []
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            data = json.load(f)
    except json.JSONDecodeError as e:
        return [f"Invalid JSON: {e}"]
    except Exception as e:
        return [f"Error reading file: {e}"]
    
    # Validate against schema
    try:
        jsonschema.validate(data, PACKAGE_SCHEMA)
    except jsonschema.ValidationError as e:
        errors.append(f"Schema validation error: {e.message}")
    
    # Additional validation rules
    if 'name' in data:
        expected_filename = f"{data['name']}.json"
        if file_path.name != expected_filename:
            errors.append(f"Filename {file_path.name} doesn't match package name {data['name']}")
    
    # Check cross-platform mappings
    if 'cross_platform' in data:
        platforms = data['cross_platform']
        total_managers = 0
        
        for platform, managers in platforms.items():
            total_managers += len(managers)
            
        if total_managers < 2:
            errors.append("Package should support at least 2 package managers")
    
    return errors

def validate_all_packages() -> bool:
    """Validate all package files in the repository"""
    packages_dir = Path("packages")
    total_files = 0
    total_errors = 0
    
    print("🔍 Validating package metadata...")
    
    for package_file in packages_dir.rglob("*.json"):
        total_files += 1
        errors = validate_package(package_file)
        
        if errors:
            total_errors += len(errors)
            print(f"❌ {package_file}:")
            for error in errors:
                print(f"   • {error}")
        else:
            print(f"✅ {package_file}")
    
    print(f"\n📊 Validation Results:")
    print(f"   Files processed: {total_files}")
    print(f"   Errors found: {total_errors}")
    
    if total_errors == 0:
        print("🎉 All package metadata is valid!")
        return True
    else:
        print(f"💥 {total_errors} errors found. Please fix them before proceeding.")
        return False

def main():
    """Main validation function"""
    os.chdir(Path(__file__).parent.parent)
    
    if validate_all_packages():
        sys.exit(0)
    else:
        sys.exit(1)

if __name__ == "__main__":
    main()