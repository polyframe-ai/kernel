# Changelog

All notable changes to the Polyframe Kernel will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2025-10-11
### Added
- Initial CI/CD pipeline with GitHub Actions
- Automated release workflow for multi-platform builds
- Release script for version bumping and validation
- Security audit in CI pipeline

### Changed
- Updated package metadata in Cargo.toml
- Improved `bump_version.sh` to automatically bump version numbers and update CHANGELOG.md
- Added retry mode to `bump_version.sh` for re-attempting failed version bumps

### Fixed
- Cross-platform compatibility in `comparator` tests (Windows CI support)
- Security audit job now uses direct `cargo-audit` installation
- Tests now use `tempfile` crate for cross-platform temporary file handling
- GitHub release permissions (403 errors) by adding explicit `contents: write` permissions


## [0.1.0] - 2025-10-11

### Added
- Initial release of Polyframe Kernel
- OpenSCAD-compatible parser using Pest
- Core geometric primitives (cube, sphere, cylinder)
- CSG operations (union, difference, intersection)
- Boolean operations with robust geometry handling
- Parallel evaluation support for performance
- STL, 3MF, glTF, and STEP export formats
- Comprehensive evaluation harness for I/O equivalence testing
- Performance benchmarking infrastructure
- FFI bindings for WASM and Node.js (via NAPI)
- CLI tool for processing OpenSCAD files
- Incremental evaluation with dependency tracking
- Mesh analytics and bounding box calculations
- Multi-threaded boolean operations
- Example files demonstrating primitives and operations

### Performance
- Optimized CSG operations with spatial indexing
- Parallel evaluation for independent operations
- Efficient memory management with specialized allocators

### Documentation
- Comprehensive README with usage examples
- Performance documentation
- Contributing guidelines
- Security policy
- Third-party license notices
- Provenance documentation

[Unreleased]: https://github.com/polyframe-ai/kernel/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/polyframe-ai/kernel/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/polyframe-ai/kernel/releases/tag/v0.1.0

