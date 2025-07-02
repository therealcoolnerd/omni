#!/bin/bash
set -euo pipefail

# Omni Staging Environment Test Script
# This script runs comprehensive tests in the staging environment

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
LOG_FILE="/tmp/omni-staging-test.log"
TEST_RESULTS_DIR="/tmp/omni-test-results"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'  
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1" | tee -a "$LOG_FILE"
}

success() {
    echo -e "${GREEN}✓${NC} $1" | tee -a "$LOG_FILE"
    ((PASSED_TESTS++))
}

error() {
    echo -e "${RED}✗${NC} $1" | tee -a "$LOG_FILE"
    ((FAILED_TESTS++))
}

warning() {
    echo -e "${YELLOW}⚠${NC} $1" | tee -a "$LOG_FILE"
}

run_test() {
    local test_name="$1"
    local test_command="$2"
    local expected_exit_code="${3:-0}"
    
    ((TOTAL_TESTS++))
    log "Running test: $test_name"
    
    if eval "$test_command" &>> "$LOG_FILE"; then
        local actual_exit_code=$?
        if [ "$actual_exit_code" -eq "$expected_exit_code" ]; then
            success "$test_name"
            return 0
        else
            error "$test_name (exit code: expected $expected_exit_code, got $actual_exit_code)"
            return 1
        fi
    else
        error "$test_name (command failed)"
        return 1
    fi
}

setup_test_environment() {
    log "Setting up test environment..."
    
    # Create test results directory
    mkdir -p "$TEST_RESULTS_DIR"
    
    # Clear log file
    > "$LOG_FILE"
    
    # Check if omni binary exists
    if [ ! -f "$PROJECT_ROOT/target/release/omni" ]; then
        error "Omni binary not found. Please run 'cargo build --release' first."
        exit 1
    fi
    
    # Add binary to PATH for tests
    export PATH="$PROJECT_ROOT/target/release:$PATH"
    
    success "Test environment setup complete"
}

test_smoke_tests() {
    log "Running smoke tests..."
    
    run_test "Version check" "omni --version"
    run_test "Help command" "omni --help"
    run_test "Config show" "omni config show"
}

test_basic_functionality() {
    log "Running basic functionality tests..."
    
    run_test "Mock search" "omni --mock search firefox"
    run_test "Mock install" "omni --mock install curl"  
    run_test "Mock history" "omni --mock history show"
    run_test "Mock remove" "omni --mock remove curl"
}

test_version_tiers() {
    log "Testing version tiers..."
    
    # Test Lite features
    run_test "Lite: Basic install" "omni --mock install vim"
    run_test "Lite: Search" "omni --mock search editor"
    
    # Test Core features (if available)
    if omni --help | grep -q "snapshot"; then
        run_test "Core: Snapshot create" "omni --mock snapshot create test-snapshot-$(date +%s)"
        run_test "Core: Snapshot list" "omni --mock snapshot list"
        
        if [ -f "$PROJECT_ROOT/tests/data/test_manifest.yaml" ]; then
            run_test "Core: Manifest install" "omni --mock manifest install $PROJECT_ROOT/tests/data/test_manifest.yaml"
        fi
    else
        warning "Core features not available in this build"
    fi
    
    # Test Enterprise features (if available) 
    if omni --help | grep -q "gui"; then
        run_test "Enterprise: GUI version" "omni gui --version"
    else
        warning "GUI features not available in this build"
    fi
    
    if omni --help | grep -q "transaction"; then
        run_test "Enterprise: Transaction begin" "omni --mock transaction begin"
    else
        warning "Transaction features not available in this build"  
    fi
}

test_package_managers() {
    log "Testing package manager detection..."
    
    # Test package manager availability detection
    run_test "Package manager detection" "omni config show | grep -E '(apt|brew|winget|chocolatey|snap|flatpak|dnf|pacman|zypper)'"
}

test_performance() {
    log "Running performance tests..."
    
    local start_time
    local end_time
    local duration
    
    # Test startup time
    start_time=$(date +%s%N)
    if omni --version > /dev/null 2>&1; then
        end_time=$(date +%s%N)
        duration=$(( (end_time - start_time) / 1000000 )) # Convert to milliseconds
        
        if [ "$duration" -lt 500 ]; then
            success "Startup time: ${duration}ms (under 500ms threshold)"
        else
            error "Startup time: ${duration}ms (exceeds 500ms threshold)"
        fi
    else
        error "Startup time test failed"
    fi
    
    # Test search performance  
    start_time=$(date +%s%N)
    if omni --mock search firefox > /dev/null 2>&1; then
        end_time=$(date +%s%N)
        duration=$(( (end_time - start_time) / 1000000 ))
        
        if [ "$duration" -lt 2000 ]; then
            success "Search time: ${duration}ms (under 2000ms threshold)"
        else
            error "Search time: ${duration}ms (exceeds 2000ms threshold)"
        fi
    else
        error "Search performance test failed"
    fi
}

test_error_handling() {
    log "Testing error handling..."
    
    run_test "Invalid command" "omni invalid-command-xyz" 1
    run_test "Invalid package" "omni --mock install non-existent-package-xyz" 1
    run_test "Invalid config" "omni config set invalid.setting invalid.value" 1
}

test_concurrent_operations() {
    log "Testing concurrent operations..."
    
    # Run multiple search operations in parallel
    local pids=()
    for i in {1..5}; do
        omni --mock search "test-package-$i" &> "/tmp/omni-concurrent-$i.log" &
        pids+=($!)
    done
    
    # Wait for all background processes
    local failed=0
    for pid in "${pids[@]}"; do
        if ! wait "$pid"; then
            ((failed++))
        fi
    done
    
    if [ "$failed" -eq 0 ]; then
        success "Concurrent operations (5 parallel searches)"
    else
        error "Concurrent operations ($failed/5 searches failed)"
    fi
    
    # Clean up
    rm -f /tmp/omni-concurrent-*.log
}

generate_test_report() {
    log "Generating test report..."
    
    local report_file="$TEST_RESULTS_DIR/staging-test-report-$(date +%Y%m%d-%H%M%S).json"
    
    cat > "$report_file" << EOF
{
  "test_run": {
    "timestamp": "$(date -Iseconds)",
    "environment": "staging",
    "total_tests": $TOTAL_TESTS,
    "passed_tests": $PASSED_TESTS,
    "failed_tests": $FAILED_TESTS,
    "success_rate": $(echo "scale=2; $PASSED_TESTS * 100 / $TOTAL_TESTS" | bc -l 2>/dev/null || echo "0"),
    "log_file": "$LOG_FILE"
  },
  "system_info": {
    "os": "$(uname -s)",
    "os_version": "$(uname -r)", 
    "architecture": "$(uname -m)",
    "hostname": "$(hostname)"
  },
  "omni_version": "$(omni --version 2>/dev/null || echo 'unknown')"
}
EOF
    
    log "Test report saved to: $report_file"
}

main() {
    log "Starting Omni staging environment tests..."
    
    setup_test_environment
    
    test_smoke_tests
    test_basic_functionality  
    test_version_tiers
    test_package_managers
    test_performance
    test_error_handling
    test_concurrent_operations
    
    generate_test_report
    
    log "Test execution completed"
    log "Results: $PASSED_TESTS passed, $FAILED_TESTS failed out of $TOTAL_TESTS total tests"
    
    if [ "$FAILED_TESTS" -eq 0 ]; then
        success "All tests passed! ✨"
        exit 0
    else
        error "Some tests failed. Check the logs for details."
        exit 1
    fi
}

# Handle script termination
trap 'log "Test script interrupted"' INT TERM

# Check for required tools
for cmd in bc; do
    if ! command -v "$cmd" &> /dev/null; then
        warning "$cmd not found. Some features may not work properly."
    fi
done

# Run main function
main "$@"