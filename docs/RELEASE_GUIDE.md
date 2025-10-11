# Release Guide for Polyframe Kernel

This guide provides step-by-step instructions for creating and publishing releases of the Polyframe Kernel.

## Quick Start

For standard releases, use the automated script:

```bash
# 1. Update version in Cargo.toml
# 2. Update CHANGELOG.md
# 3. Run the release script
./scripts/bump_version.sh
```

## Detailed Release Process

### 1. Prepare the Release

The release script handles most of this automatically!

#### Version Numbering

Follow [Semantic Versioning](https://semver.org/):
- **MAJOR** (x.0.0) - Breaking API changes
- **MINOR** (0.x.0) - New features, backward compatible
- **PATCH** (0.0.x) - Bug fixes, backward compatible

#### Update Changelog

Before running the script, add your changes to the `[Unreleased]` section:

```markdown
## [Unreleased]

### Added
- New feature description

### Changed
- Changes to existing functionality

### Fixed
- Bug fixes
```

**The script will automatically:**
- Move `[Unreleased]` content to a new version section
- Add the version number and date
- Update version comparison links
- Keep `[Unreleased]` section empty for future changes

### 2. Run the Bump Script

Run the script with the bump type:

```bash
# Interactive mode (asks you to choose)
./scripts/bump_version.sh

# Or specify directly
./scripts/bump_version.sh patch   # 0.1.0 → 0.1.1
./scripts/bump_version.sh minor   # 0.1.0 → 0.2.0
./scripts/bump_version.sh major   # 0.1.0 → 1.0.0
```

**The script will:**
1. ✅ Show current version and calculate new version
2. ✅ Update `Cargo.toml` with new version
3. ✅ Update `Cargo.lock`
4. ✅ **Automatically update `CHANGELOG.md`** (move Unreleased → versioned)
5. ✅ Check code formatting (`cargo fmt`)
6. ✅ Run linting (`cargo clippy`)
7. ✅ Run all tests (`cargo test`)
8. ✅ Validate crates.io package (`cargo publish --dry-run`)
9. ✅ Build release binary
10. ✅ Create commit: "Release vX.X.X"
11. ✅ Create git tag
12. ✅ Ask if you want to push

**If validation fails:**
- The script will stop and show the error
- Fix the reported issues
- Reset with: `git reset --hard HEAD~1 && git tag -d vX.X.X`
- Run the script again

### 3. Trigger Release

When prompted by the script, choose to push:

```
Push commit and tag now? (y/N) y
```

This pushes both the commit and tag, triggering the release workflow.

Or push manually later:

```bash
git push origin main
git push origin v0.2.0
```

### 4. Monitor Release Workflow

1. Go to [GitHub Actions](https://github.com/polyframe-ai/kernel/actions)
2. Watch the "Release" workflow
3. The workflow will:
   - Run all tests
   - Build binaries for 5 platforms
   - Create GitHub Release
   - Publish to crates.io

**Expected duration:** 15-25 minutes

### 5. Verify Release

#### Check GitHub Release

1. Visit [Releases](https://github.com/polyframe-ai/kernel/releases)
2. Verify the new release is published
3. Confirm binaries are attached:
   - `polyframe-0.2.0-x86_64-unknown-linux-gnu.tar.gz`
   - `polyframe-0.2.0-x86_64-unknown-linux-musl.tar.gz`
   - `polyframe-0.2.0-x86_64-apple-darwin.tar.gz`
   - `polyframe-0.2.0-aarch64-apple-darwin.tar.gz`
   - `polyframe-0.2.0-x86_64-pc-windows-msvc.zip`
   - `SHA256SUMS.txt`

#### Check crates.io

1. Visit [crates.io/crates/polyframe](https://crates.io/crates/polyframe)
2. Verify the new version is published
3. Test installation:
   ```bash
   cargo install polyframe
   polyframe --version
   ```

### 6. Announce Release

Consider announcing the release:
- Update project website
- Post to social media
- Notify users/community
- Update dependent projects

## Emergency: Reverting a Release

### If the tag hasn't been pushed yet:

```bash
# Delete local tag
git tag -d v0.2.0
```

### If the tag was pushed but release hasn't completed:

```bash
# Delete local tag
git tag -d v0.2.0

# Delete remote tag
git push origin :refs/tags/v0.2.0

# Cancel the GitHub Actions workflow manually
```

### If the release is on crates.io:

**Note:** You cannot un-publish from crates.io, but you can yank:

```bash
cargo yank --vers 0.2.0
```

This prevents new projects from depending on it, but existing users can still use it.

## Hotfix Release Process

For urgent bug fixes:

1. Create a hotfix branch from the tag:
   ```bash
   git checkout -b hotfix/0.2.1 v0.2.0
   ```

2. Make the minimal fix
3. Update version to patch (e.g., 0.2.0 → 0.2.1)
4. Update CHANGELOG.md
5. Run `./scripts/bump_version.sh`
6. Merge back to main:
   ```bash
   git checkout main
   git merge hotfix/0.2.1
   ```

## Pre-Release (Beta/RC) Process

For testing before final release:

1. Use pre-release version numbers:
   ```toml
   version = "0.2.0-beta.1"
   # or
   version = "0.2.0-rc.1"
   ```

2. Tag with pre-release:
   ```bash
   git tag v0.2.0-beta.1
   git push origin v0.2.0-beta.1
   ```

3. The release workflow will mark it as "pre-release" on GitHub

4. Users can install with:
   ```bash
   cargo install polyframe --version 0.2.0-beta.1
   ```

## Troubleshooting

### Issue: "Tag already exists"

**Solution:** The version in Cargo.toml matches an existing tag. Increment the version.

### Issue: "Tests failed"

**Solution:** The release is blocked. Fix failing tests before creating a new release.

### Issue: "cargo publish failed"

**Possible causes:**
- Version already published to crates.io → Increment version
- Missing `CARGO_REGISTRY_TOKEN` → Add to GitHub secrets
- Package validation errors → Fix reported issues

### Issue: "Binary build failed on macOS"

**Solution:** 
- Check if dependencies support the target platform
- May need to add platform-specific dependencies in `Cargo.toml`

### Issue: "Workflow stuck on 'Publish to crates.io'"

**Possible causes:**
- Network issues (rare)
- Rate limiting (very rare)
- Invalid token

**Solution:** 
- Check workflow logs for details
- Verify `CARGO_REGISTRY_TOKEN` is valid
- Can manually publish: `cargo publish`

## Advanced: Manual Release

If you need to release without the automation:

```bash
# 1. Update version and changelog
# 2. Commit changes
git add Cargo.toml CHANGELOG.md
git commit -m "Release v0.2.0"

# 3. Create tag
git tag -a v0.2.0 -m "Release version 0.2.0"

# 4. Push everything
git push origin main
git push origin v0.2.0

# 5. Build and publish manually (if needed)
cargo build --release
cargo publish
```

## Checklist

Before releasing, ensure:

- [ ] Version updated in `Cargo.toml`
- [ ] `CHANGELOG.md` has entry for this version
- [ ] All tests pass locally
- [ ] No uncommitted changes
- [ ] Code is formatted (`cargo fmt`)
- [ ] No Clippy warnings (`cargo clippy`)
- [ ] Documentation is up to date
- [ ] Examples work with new version
- [ ] `CARGO_REGISTRY_TOKEN` secret is set in GitHub

After releasing, verify:

- [ ] GitHub Release is created
- [ ] All binary artifacts are attached
- [ ] `SHA256SUMS.txt` is present
- [ ] crates.io shows new version
- [ ] `cargo install polyframe` works
- [ ] Release notes are accurate

## Help

For issues or questions about the release process:
- Check [CI/CD Documentation](./CI_CD.md)
- Review [GitHub Actions logs](https://github.com/polyframe-ai/kernel/actions)
- Open an issue with the `release` label

