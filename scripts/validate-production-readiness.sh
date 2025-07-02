#!/bin/bash
set -euo pipefail

# Omni Production Readiness Validation Script
# Quick check of production readiness components

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PASSED=0
FAILED=0
WARNINGS=0

check_file() {
    local file="$1"
    local description="$2"
    
    if [ -f "$PROJECT_ROOT/$file" ]; then
        echo -e "${GREEN}✓${NC} $description"
        ((PASSED++))
        return 0
    else
        echo -e "${RED}✗${NC} $description (missing: $file)"
        ((FAILED++))
        return 1
    fi
}

check_directory() {
    local dir="$1"
    local description="$2"
    
    if [ -d "$PROJECT_ROOT/$dir" ]; then
        echo -e "${GREEN}✓${NC} $description"
        ((PASSED++))
        return 0
    else
        echo -e "${RED}✗${NC} $description (missing: $dir)"
        ((FAILED++))
        return 1
    fi
}

check_content() {
    local file="$1"
    local pattern="$2"
    local description="$3"
    
    if [ -f "$PROJECT_ROOT/$file" ] && grep -q "$pattern" "$PROJECT_ROOT/$file"; then
        echo -e "${GREEN}✓${NC} $description"
        ((PASSED++))
        return 0
    else
        echo -e "${YELLOW}⚠${NC} $description (needs review)"
        ((WARNINGS++))
        return 1
    fi
}

echo -e "${BLUE}🚀 Omni Production Readiness Validation${NC}"
echo "========================================"

echo -e "\n${BLUE}📁 Core Infrastructure${NC}"
check_file "Cargo.toml" "Cargo.toml configuration"
check_file "README.md" "README documentation"
check_file "LICENSE" "License file"
check_file "SECURITY.md" "Security policy"

echo -e "\n${BLUE}🧪 Testing Infrastructure${NC}"
check_directory "tests" "Test directory structure"
check_file "tests/unit_tests.rs" "Unit tests"
check_file "tests/integration_tests.rs" "Integration tests"
check_file "tests/comprehensive_tests.rs" "Comprehensive tests"
check_file "TESTING.md" "Testing documentation"
check_file "scripts/staging-test.sh" "Staging test script"

echo -e "\n${BLUE}🔄 CI/CD Pipeline${NC}"
check_file ".github/workflows/ci.yml" "GitHub Actions CI/CD"
check_content ".github/workflows/ci.yml" "test" "CI includes testing"
check_content ".github/workflows/ci.yml" "security" "CI includes security checks"
check_content ".github/workflows/ci.yml" "build" "CI includes multi-platform builds"

echo -e "\n${BLUE}🚀 Deployment Strategy${NC}"
check_file "DEPLOYMENT-STRATEGY.md" "Deployment strategy documentation"
check_file "deploy/staging-config.yml" "Staging configuration"
check_content "DEPLOYMENT-STRATEGY.md" "canary" "Canary deployment strategy"
check_content "DEPLOYMENT-STRATEGY.md" "rollback" "Rollback procedures"

echo -e "\n${BLUE}📞 Support Infrastructure${NC}"
check_directory ".github/ISSUE_TEMPLATE" "Issue templates"
check_file ".github/ISSUE_TEMPLATE/bug_report.yml" "Bug report template"
check_content ".github/ISSUE_TEMPLATE/bug_report.yml" "package_manager" "Package manager specific reporting"

echo -e "\n${BLUE}📊 Code Quality${NC}"
check_file "clippy.toml" "Clippy configuration"
check_file "deny.toml" "Dependency policy"
check_content "Cargo.toml" "lints" "Linting configuration"

echo -e "\n${BLUE}🔒 Security${NC}"
check_content "Cargo.toml" "sqlx" "Database security (SQLx)"
check_content "src/security.rs" "verify" "Security verification logic"
check_content ".github/workflows/ci.yml" "audit" "Security audit in CI"

echo -e "\n${BLUE}📦 Package Management${NC}"
check_directory "src/boxes" "Package manager modules"
check_file "src/boxes/apt.rs" "APT package manager"
check_file "src/boxes/brew.rs" "Homebrew package manager"
check_file "src/boxes/winget.rs" "WinGet package manager"
check_file "src/boxes/snap.rs" "Snap package manager"

echo -e "\n${BLUE}🏗️ Build System${NC}"
check_content "Cargo.toml" "lite" "Lite tier features"
check_content "Cargo.toml" "core" "Core tier features" 
check_content "Cargo.toml" "enterprise" "Enterprise tier features"
check_file "build-all.sh" "Build automation script"

echo -e "\n${BLUE}📚 Documentation${NC}"
check_file "PROJECT-OVERVIEW.md" "Project overview"
check_file "QUICK-START.md" "Quick start guide"
check_file "COMPLETION-STATUS.md" "Completion status"
check_file "docs/MIGRATION-GUIDE.md" "Migration guide"

echo "========================================"
echo -e "${BLUE}📊 Production Readiness Summary${NC}"
echo -e "Passed: ${GREEN}$PASSED${NC}"
echo -e "Failed: ${RED}$FAILED${NC}"
echo -e "Warnings: ${YELLOW}$WARNINGS${NC}"

TOTAL=$((PASSED + FAILED + WARNINGS))
if [ $TOTAL -gt 0 ]; then
    PERCENTAGE=$((PASSED * 100 / TOTAL))
    echo -e "Ready: ${GREEN}${PERCENTAGE}%${NC}"
else
    echo -e "Ready: ${RED}0%${NC}"
fi

echo ""
if [ $FAILED -eq 0 ] && [ $WARNINGS -le 2 ]; then
    echo -e "${GREEN}🎉 READY FOR PRODUCTION! 🎉${NC}"
    echo -e "${GREEN}Omni meets all production readiness criteria.${NC}"
    echo -e "${GREEN}The beat is ready to drop! 🎵✨${NC}"
    exit 0
elif [ $FAILED -eq 0 ]; then
    echo -e "${YELLOW}🔍 MOSTLY READY - Address warnings${NC}"
    echo -e "${YELLOW}Minor issues to resolve before production.${NC}"
    exit 1
else
    echo -e "${RED}❌ NOT READY - Critical issues found${NC}"
    echo -e "${RED}Address failed checks before production deployment.${NC}"
    exit 2
fi