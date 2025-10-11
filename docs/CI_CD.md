# CI/CD Pipeline Documentation

This document describes the Continuous Integration and Continuous Deployment (CI/CD) setup for the Polyframe Kernel project.

## Overview

The Polyframe Kernel uses GitHub Actions for automated testing, building, and releasing. The CI/CD pipeline consists of three main workflows:

1. **CI Workflow** (`.github/workflows/ci.yml`) - Runs on every push and pull request
2. **Release Workflow** (`.github/workflows/release.yml`) - Triggers on version tags
3. **Equivalence Testing** (`.github/workflows/verify-equivalence.yml`) - Nightly and on-demand validation

## CI Workflow

**Trigger:** Push and pull requests to `main` and `develop` branches

### Jobs

#### 1. Format Check (`fmt`)
- Validates code formatting with `rustfmt`
- Ensures consistent code style across the project
- **Fails if:** Code is not properly formatted

#### 2. Clippy Lints (`clippy`)
- Runs Clippy static analysis
- Treats all warnings as errors (`-D warnings`)
- **Fails if:** Any linting issues are found

#### 3. Test Suite (`test`)
- **Matrix:** Runs on Ubuntu, macOS, and Windows
- Executes all unit tests, integration tests, and doc tests
- Uses Cargo caching for faster builds
- **Fails if:** Any test fails on any platform

#### 4. Integration Tests (`integration`)
- Runs the evaluation harness on example files
- Validates end-to-end functionality
- Uploads results as artifacts (retained for 7 days)
- **Runs after:** Test suite passes

#### 5. Performance Benchmarks (`benchmark`)
- **Trigger:** Only on push to `main` branch
- Runs Criterion benchmarks
- Uploads results as artifacts (retained for 30 days)
- Provides performance regression detection

#### 6. Security Audit (`security`)
- Uses `cargo-audit` to check for known vulnerabilities
- Scans dependencies for security advisories
- **Fails if:** Critical vulnerabilities are found

### Caching Strategy

All jobs use GitHub Actions cache for Cargo dependencies:
- Cargo binary cache
- Registry index and cache
- Git database
- Build artifacts in `target/`

Cache keys are based on `Cargo.lock` hash, ensuring fresh builds when dependencies change.

## Release Workflow

**Trigger:** Git tags matching `v*.*.*` (e.g., `v0.1.0`)

### Process

1. **Create Release**
   - Extracts changelog from `CHANGELOG.md` for the tagged version
   - Creates a GitHub Release with release notes
   - Provides upload URL for release artifacts

2. **Pre-Release Tests**
   - Runs format check, Clippy, and full test suite
   - Builds release binary
   - Runs integration tests with evaluation harness
   - **Blocks release if:** Any check fails

3. **Multi-Platform Builds**
   - **Platforms:**
     - `x86_64-unknown-linux-gnu` (Ubuntu, glibc)
     - `x86_64-unknown-linux-musl` (Alpine Linux, static binary)
     - `x86_64-apple-darwin` (macOS Intel)
     - `aarch64-apple-darwin` (macOS Apple Silicon)
     - `x86_64-pc-windows-msvc` (Windows)
   
   - **For each platform:**
     - Builds optimized release binary
     - Creates compressed archive (`.tar.gz` or `.zip`)
     - Generates SHA-256 checksum
     - Uploads to GitHub Release

4. **Publish Checksums**
   - Combines all individual checksums into `SHA256SUMS.txt`
   - Attaches to GitHub Release for easy verification

5. **Publish to crates.io**
   - Validates the build
   - Publishes package to crates.io
   - **Requires:** `CARGO_REGISTRY_TOKEN` secret

## Secrets Configuration

### Required Secrets

Add these secrets in your GitHub repository settings (`Settings > Secrets and variables > Actions`):

#### `CARGO_REGISTRY_TOKEN`
**Purpose:** Authenticate with crates.io for publishing

**How to obtain:**
1. Log in to [crates.io](https://crates.io/)
2. Go to **Account Settings** → **API Tokens**
3. Click **New Token**
4. Name it (e.g., "GitHub Actions Release")
5. Copy the generated token
6. Add to GitHub repository secrets

**Permissions:** Must have publish access to the `polyframe` crate

## Making a Release

### Using the Release Script (Recommended)

The project includes an automated release script that handles all validation steps:

```bash
# 1. Update version in Cargo.toml
# Edit Cargo.toml and set the new version number

# 2. Update CHANGELOG.md
# Add a new section for the version with changes

# 3. Run the release script
./scripts/release.sh
```

The script will:
- ✅ Run `cargo fmt -- --check`
- ✅ Run `cargo clippy -- -D warnings`
- ✅ Run `cargo test --all`
- ✅ Run `cargo publish --dry-run`
- ✅ Build release binary
- ✅ Create a git tag for the version
- ✅ Optionally push the tag to trigger release

### Manual Release Process

If you prefer manual control:

```bash
# 1. Update version
# Edit Cargo.toml: version = "0.2.0"

# 2. Update CHANGELOG.md
# Add release notes for the new version

# 3. Commit changes
git add Cargo.toml CHANGELOG.md
git commit -m "Bump version to 0.2.0"

# 4. Create and push tag
git tag -a v0.2.0 -m "Release version 0.2.0"
git push origin v0.2.0
```

### Post-Release

Once the tag is pushed:

1. **Monitor GitHub Actions**: Check the [Actions tab](https://github.com/polyframe-ai/kernel/actions) for workflow progress
2. **Verify Release**: Check that binaries are uploaded to the GitHub Release
3. **Verify crates.io**: Confirm the package appears on [crates.io/crates/polyframe](https://crates.io/crates/polyframe)
4. **Test Installation**:
   ```bash
   cargo install polyframe
   polyframe --version
   ```

## Artifact Retention

- **CI test results**: 7 days
- **Benchmark results**: 30 days
- **Evaluation reports**: 30 days
- **Release binaries**: Permanent (attached to GitHub Release)

## Troubleshooting

### Release Workflow Fails

**Issue:** Pre-release tests fail
- **Solution:** The release is blocked to prevent publishing broken code. Fix the failing tests and create a new tag.

**Issue:** `cargo publish` fails with "already uploaded"
- **Solution:** Version already exists on crates.io. Update version in `Cargo.toml` and create a new tag.

**Issue:** Missing `CARGO_REGISTRY_TOKEN`
- **Solution:** Add the token as a repository secret (see Secrets Configuration above)

### CI Workflow Slow

**Issue:** Cache not working
- **Solution:** Check that `Cargo.lock` is committed to the repository. GitHub Actions cache is based on this file's hash.

**Issue:** Windows builds timeout
- **Solution:** This is rare but can happen with large dependency trees. Contact GitHub Support or reduce dependencies.

### Binary Verification

To verify downloaded binaries:

```bash
# Download SHA256SUMS.txt from the release
curl -LO https://github.com/polyframe-ai/kernel/releases/download/v0.1.0/SHA256SUMS.txt

# Verify a downloaded binary
sha256sum -c SHA256SUMS.txt --ignore-missing
```

## Maintenance

### Updating Workflows

When modifying workflows:

1. Test changes on a feature branch
2. Verify workflow runs successfully
3. Merge to `main` only after validation

### Dependency Updates

The security audit job will notify you of vulnerable dependencies. To update:

```bash
cargo update
cargo test --all
cargo clippy --all-targets --all-features
```

Then commit the updated `Cargo.lock`.

### Adding New Platforms

To support additional platforms, edit `.github/workflows/release.yml`:

```yaml
matrix:
  include:
    # Add new platform here
    - os: ubuntu-latest
      target: aarch64-unknown-linux-gnu
      archive: tar.gz
```

Ensure the Rust toolchain supports the target and any required cross-compilation tools are installed.

## Best Practices

1. **Always run tests locally** before pushing
2. **Update CHANGELOG.md** with every notable change
3. **Use semantic versioning** for releases
4. **Don't force-push** to `main` - it breaks CI
5. **Review workflow logs** if a build fails
6. **Keep dependencies updated** to avoid security issues
7. **Test release script** in a separate branch first

## Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [cargo-audit](https://github.com/RustSec/rustsec/tree/main/cargo-audit)
- [softprops/action-gh-release](https://github.com/softprops/action-gh-release)
- [Semantic Versioning](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)

## Support

For issues with the CI/CD pipeline, please:
1. Check the [Actions tab](https://github.com/polyframe-ai/kernel/actions) for error logs
2. Review this documentation
3. Open an issue with the `ci/cd` label

