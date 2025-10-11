#!/usr/bin/env bash
# Polyframe Kernel - Version Bump and Release Script
# 
# This script automates the release process by:
# 1. Bumping the version number (major, minor, or patch)
# 2. Updating Cargo.toml and Cargo.lock
# 3. Running all quality checks (tests, formatting, linting)
# 4. Committing changes and creating a git tag
# 5. Pushing to trigger the release workflow
#
# Usage:
#   ./scripts/bump_version.sh [major|minor|patch]
#   ./scripts/bump_version.sh           # Interactive mode
#
# Prerequisites:
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
command -v cargo >/dev/null 2>&1 || error "cargo is not installed. Please install Rust toolchain."
command -v git >/dev/null 2>&1 || error "git is not installed."

# Check if we're on a clean git state
if [ -n "$(git status --porcelain)" ]; then
    warning "Working directory is not clean. Please commit or stash changes first."
    git status --short
    exit 1
fi

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
if [ -z "$CURRENT_VERSION" ]; then
    error "Could not extract version from Cargo.toml"
fi

info "Current version: ${CURRENT_VERSION}"

# Parse current version
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"

# Determine bump type
BUMP_TYPE=$1
if [ -z "$BUMP_TYPE" ]; then
    echo ""
    echo "Select version bump type:"
    echo "  1) patch (${MAJOR}.${MINOR}.$((PATCH + 1))) - Bug fixes"
    echo "  2) minor (${MAJOR}.$((MINOR + 1)).0) - New features (backward compatible)"
    echo "  3) major ($((MAJOR + 1)).0.0) - Breaking changes"
    echo ""
    read -p "Enter choice [1-3]: " choice
    
    case $choice in
        1) BUMP_TYPE="patch" ;;
        2) BUMP_TYPE="minor" ;;
        3) BUMP_TYPE="major" ;;
        *) error "Invalid choice" ;;
    esac
fi

# Calculate new version
case $BUMP_TYPE in
    patch)
        NEW_VERSION="${MAJOR}.${MINOR}.$((PATCH + 1))"
        ;;
    minor)
        NEW_VERSION="${MAJOR}.$((MINOR + 1)).0"
        ;;
    major)
        NEW_VERSION="$((MAJOR + 1)).0.0"
        ;;
    *)
        error "Invalid bump type. Use: major, minor, or patch"
        ;;
esac

echo ""
info "Bumping version: ${CURRENT_VERSION} → ${NEW_VERSION}"
echo ""

# Confirm
read -p "Continue with version bump? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    info "Version bump cancelled"
    exit 0
fi

# Update Cargo.toml
info "Updating Cargo.toml..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/^version = \"${CURRENT_VERSION}\"/version = \"${NEW_VERSION}\"/" Cargo.toml
else
    # Linux
    sed -i "s/^version = \"${CURRENT_VERSION}\"/version = \"${NEW_VERSION}\"/" Cargo.toml
fi
success "Cargo.toml updated"

# Update Cargo.lock
info "Updating Cargo.lock..."
cargo update -p polyframe --quiet
success "Cargo.lock updated"

# Update CHANGELOG.md
echo ""
info "Updating CHANGELOG.md..."

if [ ! -f "CHANGELOG.md" ]; then
    warning "CHANGELOG.md not found, skipping changelog update"
else
    # Check if there's an Unreleased section with content
    if grep -q "## \[Unreleased\]" CHANGELOG.md; then
        # Create a temporary file
        TEMP_FILE=$(mktemp)
        
        # Get today's date
        TODAY=$(date +%Y-%m-%d)
        
        # Process the CHANGELOG
        awk -v version="$NEW_VERSION" -v date="$TODAY" -v oldversion="$CURRENT_VERSION" '
        /^## \[Unreleased\]/ {
            print $0
            print ""
            # Skip the unreleased section and capture it
            in_unreleased = 1
            next
        }
        in_unreleased && /^## / {
            # We hit the next section, insert the new version section
            print "## [" version "] - " date
            print unreleased_content
            in_unreleased = 0
            print $0
            next
        }
        in_unreleased {
            # Accumulate unreleased content (skip empty lines at start)
            if (length(unreleased_content) > 0 || NF > 0) {
                unreleased_content = unreleased_content $0 "\n"
            }
            next
        }
        /^\[Unreleased\]:/ {
            # Update the version comparison links
            print "[Unreleased]: https://github.com/polyframe-ai/kernel/compare/v" version "...HEAD"
            print "[" version "]: https://github.com/polyframe-ai/kernel/compare/v" oldversion "...v" version
            next
        }
        /^\[.*\]:/ && !printed_new_link {
            # Insert new version link before other version links
            print "[" version "]: https://github.com/polyframe-ai/kernel/compare/v" oldversion "...v" version
            printed_new_link = 1
        }
        { print }
        ' CHANGELOG.md > "$TEMP_FILE"
        
        # Check if the update was successful
        if grep -q "\[${NEW_VERSION}\]" "$TEMP_FILE"; then
            mv "$TEMP_FILE" CHANGELOG.md
            success "CHANGELOG.md updated with version ${NEW_VERSION}"
        else
            warning "Could not automatically update CHANGELOG.md"
            rm "$TEMP_FILE"
            echo ""
            echo "Please manually add this section to CHANGELOG.md:"
            echo ""
            echo "## [${NEW_VERSION}] - $(date +%Y-%m-%d)"
            echo ""
            read -p "Press Enter after updating CHANGELOG.md..."
        fi
    else
        warning "No [Unreleased] section found in CHANGELOG.md"
        echo ""
        echo "Please add a section for version ${NEW_VERSION}:"
        echo ""
        echo "## [${NEW_VERSION}] - $(date +%Y-%m-%d)"
        echo ""
        read -p "Press Enter after updating CHANGELOG.md..."
    fi
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

# All checks passed, create commit and tag
echo ""
success "All pre-release checks passed!"
echo ""

# Stage changes
git add Cargo.toml Cargo.lock CHANGELOG.md

# Create commit
info "Creating release commit..."
git commit -m "Release v${NEW_VERSION}"
success "Commit created"

# Create tag
info "Creating git tag v${NEW_VERSION}..."
git tag -a "v${NEW_VERSION}" -m "Release version ${NEW_VERSION}"
success "Tag v${NEW_VERSION} created"

# Show what will be pushed
echo ""
info "Ready to push! This will:"
echo "  • Push the release commit to main"
echo "  • Push tag v${NEW_VERSION}"
echo "  • Trigger the release workflow on GitHub"
echo ""
info "The workflow will:"
echo "  • Build binaries for Linux, macOS, and Windows"
echo "  • Run integration tests"
echo "  • Create a GitHub release with binaries"
echo "  • Publish to crates.io"
echo ""

# Ask for confirmation to push
read -p "Push commit and tag now? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    info "Pushing to remote..."
    git push origin main
    git push origin "v${NEW_VERSION}"
    success "Pushed! Release workflow started."
    echo ""
    info "Monitor the release progress at:"
    echo "https://github.com/polyframe-ai/kernel/actions"
else
    info "Not pushed. You can push later with:"
    echo "    git push origin main"
    echo "    git push origin v${NEW_VERSION}"
    echo ""
    warning "To undo the local changes, run:"
    echo "    git reset --hard HEAD~1"
    echo "    git tag -d v${NEW_VERSION}"
fi

echo ""
success "Version bump complete: ${CURRENT_VERSION} → ${NEW_VERSION}"
