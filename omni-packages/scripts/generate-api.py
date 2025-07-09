#!/usr/bin/env python3
"""
Generate API endpoints from package metadata
"""

import json
import os
from pathlib import Path
from typing import Dict, List, Any
from datetime import datetime, timezone

def load_all_packages() -> List[Dict[str, Any]]:
    """Load all package metadata files"""
    packages = []
    packages_dir = Path("packages")
    
    for package_file in packages_dir.rglob("*.json"):
        try:
            with open(package_file, 'r', encoding='utf-8') as f:
                package_data = json.load(f)
                packages.append(package_data)
        except Exception as e:
            print(f"Error loading {package_file}: {e}")
    
    return packages

def generate_popular_packages(packages: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Generate popular packages API endpoint"""
    
    # Sort by popularity rank (lower number = more popular)
    popular = sorted(
        [p for p in packages if 'popularity' in p and 'rank' in p['popularity']], 
        key=lambda x: x['popularity']['rank']
    )
    
    popular_packages = []
    categories = {}
    
    for pkg in popular:
        pop_data = pkg['popularity']
        pkg_info = {
            "rank": pop_data['rank'],
            "name": pkg['name'],
            "display_name": pkg['display_name'],
            "category": pkg['category'],
            "downloads_per_month": pop_data.get('downloads_per_month', 0),
            "search_frequency": pop_data.get('search_frequency', 0),
            "cross_platform": bool(pkg.get('cross_platform'))
        }
        popular_packages.append(pkg_info)
        
        # Group by category
        category = pkg['category']
        if category not in categories:
            categories[category] = []
        categories[category].append(pkg['name'])
    
    return {
        "version": "1.0",
        "last_updated": datetime.now(timezone.utc).isoformat(),
        "total_packages": len(popular_packages),
        "popular_packages": popular_packages,
        "categories": categories
    }

def generate_cross_platform_mappings(packages: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Generate cross-platform mappings API endpoint"""
    
    mappings = {}
    reverse_mappings = {"linux": {}, "macos": {}, "windows": {}}
    
    for pkg in packages:
        if 'cross_platform' not in pkg:
            continue
            
        name = pkg['name']
        cross_platform = pkg['cross_platform']
        mappings[name] = cross_platform
        
        # Build reverse mappings
        for platform, managers in cross_platform.items():
            if platform not in reverse_mappings:
                reverse_mappings[platform] = {}
                
            for manager, package_names in managers.items():
                if manager not in reverse_mappings[platform]:
                    reverse_mappings[platform][manager] = {}
                    
                for package_name in package_names:
                    reverse_mappings[platform][manager][package_name] = name
    
    return {
        "version": "1.0",
        "last_updated": datetime.now(timezone.utc).isoformat(),
        "mappings": mappings,
        "reverse_mappings": reverse_mappings
    }

def generate_security_data(packages: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Generate security scores API endpoint"""
    
    security_scores = {}
    security_categories = {"excellent": [], "good": [], "fair": [], "poor": []}
    vulnerability_alerts = []
    
    for pkg in packages:
        if 'security' not in pkg:
            continue
            
        name = pkg['name']
        security = pkg['security']
        
        security_scores[name] = security
        
        # Categorize by security score
        score = security.get('score', 0)
        if score >= 9.0:
            security_categories["excellent"].append(name)
        elif score >= 8.0:
            security_categories["good"].append(name)
        elif score >= 6.0:
            security_categories["fair"].append(name)
        else:
            security_categories["poor"].append(name)
            
        # Check for vulnerabilities
        if security.get('vulnerabilities') and len(security['vulnerabilities']) > 0:
            vulnerability_alerts.extend(security['vulnerabilities'])
    
    return {
        "version": "1.0",
        "last_updated": datetime.now(timezone.utc).isoformat(),
        "security_scores": security_scores,
        "security_categories": security_categories,
        "vulnerability_alerts": vulnerability_alerts,
        "security_guidelines": {
            "minimum_score": 7.0,
            "recommended_score": 8.5,
            "update_frequency": "daily"
        }
    }

def generate_categories(packages: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Generate categories API endpoint"""
    
    categories = {}
    
    for pkg in packages:
        category = pkg.get('category', 'other')
        if category not in categories:
            categories[category] = []
        categories[category].append({
            "name": pkg['name'],
            "display_name": pkg['display_name'],
            "description": pkg['description']
        })
    
    return {
        "version": "1.0",
        "last_updated": datetime.now(timezone.utc).isoformat(),
        "categories": categories,
        "total_categories": len(categories),
        "total_packages": len(packages)
    }

def generate_all_packages(packages: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Generate complete packages list API endpoint"""
    
    return {
        "version": "1.0", 
        "last_updated": datetime.now(timezone.utc).isoformat(),
        "total_packages": len(packages),
        "packages": packages
    }

def main():
    """Generate all API endpoints"""
    os.chdir(Path(__file__).parent.parent)
    
    print("📦 Loading package metadata...")
    packages = load_all_packages()
    print(f"✅ Loaded {len(packages)} packages")
    
    # Create API directory structure
    api_dir = Path("api/v1/packages")
    api_dir.mkdir(parents=True, exist_ok=True)
    
    # Generate API endpoints
    endpoints = {
        "popular.json": generate_popular_packages(packages),
        "cross-platform.json": generate_cross_platform_mappings(packages),
        "security.json": generate_security_data(packages),
        "categories.json": generate_categories(packages),
        "all.json": generate_all_packages(packages)
    }
    
    print("🔄 Generating API endpoints...")
    
    for filename, data in endpoints.items():
        output_path = api_dir / filename
        
        with open(output_path, 'w', encoding='utf-8') as f:
            json.dump(data, f, indent=2, ensure_ascii=False)
        
        print(f"✅ Generated {output_path}")
    
    print("🎉 API generation complete!")

if __name__ == "__main__":
    main()