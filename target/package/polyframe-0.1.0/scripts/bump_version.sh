#!/usr/bin/env bash
# Polyframe Kernel - Version Bump and Release Script
# 
# This script automates the release process by:
# 1. Running all quality checks (tests, formatting, linting)
# 2. Validating the build for crates.io
# 3. Creating a git tag for the current version
# 4. Pushing the tag to trigger the release workflow
#
# Usage:
#   ./scripts/bump_version.sh
#
# Prerequisites:
#   - jq (for parsing Cargo metadata)
#   - cargo (Rust toolchain)
#   - git

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
error() {
    echo -e "${RED}❌ Error: $1${NC}" >&2
    exit 1
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Check prerequisites
command -v jq >/dev/null 2>&1 || error "jq is not installed. Please install it first."
command -v cargo >/dev/null 2>&1 || error "cargo is not installed. Please install Rust toolchain."
command -v git >/dev/null 2>&1 || error "git is not installed."

# Get current version from Cargo.toml
VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')
if [ -z "$VERSION" ]; then
    error "Could not extract version from Cargo.toml"
fi

info "Current version: ${VERSION}"

# Check if we're on a clean git state
if [ -n "$(git status --porcelain)" ]; then
    warning "Working directory is not clean. Please commit or stash changes first."
    git status --short
    exit 1
fi

# Check if tag already exists
if git rev-parse "v${VERSION}" >/dev/null 2>&1; then
    error "Tag v${VERSION} already exists. Please update version in Cargo.toml first."
fi

info "Running pre-release checks..."

# Step 1: Format check
echo ""
info "Step 1/5: Checking code formatting..."
if cargo fmt -- --check; then
    success "Code formatting is correct"
else
    error "Code formatting check failed. Run 'cargo fmt' to fix."
fi

# Step 2: Clippy lints
echo ""
info "Step 2/5: Running Clippy lints..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    success "Clippy checks passed"
else
    error "Clippy found issues. Please fix them before releasing."
fi

# Step 3: Run all tests
echo ""
info "Step 3/5: Running test suite..."
if cargo test --all; then
    success "All tests passed"
else
    error "Tests failed. Please fix failing tests before releasing."
fi

# Step 4: Dry-run publish to crates.io
echo ""
info "Step 4/5: Validating crates.io package..."
if cargo publish --dry-run; then
    success "Package validation successful"
else
    error "Package validation failed. Please fix issues before releasing."
fi

# Step 5: Build release binary
echo ""
info "Step 5/5: Building release binary..."
if cargo build --release; then
    success "Release build successful"
else
    error "Release build failed."
fi

# All checks passed, create tag
echo ""
info "All pre-release checks passed!"
echo ""
info "Creating git tag v${VERSION}..."

git tag -a "v${VERSION}" -m "Release version ${VERSION}"
success "Tag v${VERSION} created"

# Show what will be pushed
echo ""
info "Tag created successfully. To publish the release, run:"
echo ""
echo "    git push origin v${VERSION}"
echo ""
info "This will trigger the GitHub Actions release workflow which will:"
echo "  • Build binaries for Linux, macOS, and Windows"
echo "  • Run integration tests"
echo "  • Create a GitHub release with binaries"
echo "  • Publish to crates.io"
echo ""

# Ask for confirmation to push
read -p "Do you want to push the tag now? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    info "Pushing tag to remote..."
    git push origin "v${VERSION}"
    success "Tag pushed! Release workflow started."
    echo ""
    info "Monitor the release progress at:"
    echo "https://github.com/mihok/polyframe-kernel/actions"
else
    info "Tag not pushed. You can push it later with:"
    echo "    git push origin v${VERSION}"
    echo ""
    warning "To delete the local tag if needed, run:"
    echo "    git tag -d v${VERSION}"
fi

echo ""
success "Release preparation complete!"

