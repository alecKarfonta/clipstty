#!/bin/bash

# STT Clippy Build Script
# This script helps build and test the STT Clippy application

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    if ! command_exists cargo; then
        print_error "Rust and Cargo are not installed. Please install Rust from https://rustup.rs/"
        exit 1
    fi
    
    if ! command_exists rustc; then
        print_error "Rust compiler is not installed. Please install Rust from https://rustup.rs/"
        exit 1
    fi
    
    print_success "Prerequisites check passed"
}

# Function to clean build artifacts
clean() {
    print_status "Cleaning build artifacts..."
    cargo clean
    print_success "Clean completed"
}

# Function to check code formatting
check_format() {
    print_status "Checking code format..."
    if cargo fmt --check; then
        print_success "Code format check passed"
    else
        print_warning "Code format check failed. Run 'cargo fmt' to fix formatting issues"
        return 1
    fi
}

# Function to run linter
lint() {
    print_status "Running linter..."
    if cargo clippy -- -D warnings; then
        print_success "Linting passed"
    else
        print_warning "Linting found issues. Please fix them before proceeding"
        return 1
    fi
}

# Function to run tests
test() {
    print_status "Running tests..."
    if cargo test; then
        print_success "All tests passed"
    else
        print_error "Tests failed"
        return 1
    fi
}

# Function to build in debug mode
build_debug() {
    print_status "Building in debug mode..."
    if cargo build; then
        print_success "Debug build completed"
    else
        print_error "Debug build failed"
        return 1
    fi
}

# Function to build in release mode
build_release() {
    print_status "Building in release mode..."
    if cargo build --release; then
        print_success "Release build completed"
    else
        print_error "Release build failed"
        return 1
    fi
}

# Function to run the application
run() {
    print_status "Running STT Clippy..."
    if cargo run; then
        print_success "Application ran successfully"
    else
        print_error "Application failed to run"
        return 1
    fi
}

# Function to show help
show_help() {
    echo "STT Clippy Build Script"
    echo ""
    echo "Usage: $0 [OPTION]"
    echo ""
    echo "Options:"
    echo "  clean       Clean build artifacts"
    echo "  check       Check code format and run linter"
    echo "  test        Run all tests"
    echo "  build       Build in debug mode"
    echo "  release     Build in release mode"
    echo "  run         Build and run the application"
    echo "  all         Run all checks and build"
    echo "  help        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 build     # Build in debug mode"
    echo "  $0 release   # Build in release mode"
    echo "  $0 test      # Run tests"
    echo "  $0 all       # Run all checks and build"
}

# Main script logic
main() {
    # Check prerequisites first
    check_prerequisites
    
    case "${1:-help}" in
        clean)
            clean
            ;;
        check)
            check_format
            lint
            ;;
        test)
            test
            ;;
        build)
            build_debug
            ;;
        release)
            build_release
            ;;
        run)
            build_debug
            run
            ;;
        all)
            check_format
            lint
            test
            build_debug
            print_success "All checks and build completed successfully"
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            print_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"
