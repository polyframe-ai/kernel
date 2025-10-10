# Contributing to Polyframe Kernel

Thanks for your interest in contributing to Polyframe Kernel! We welcome issues, bug reports, feature requests, and pull requests.

## Quick Start

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Git
- (Optional) OpenSCAD for equivalence testing

### Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/mihok/polyframe-kernel.git
   cd polyframe-kernel
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run tests:
   ```bash
   cargo test
   ```

4. Run the CLI:
   ```bash
   cargo run --bin polyframe -- examples/complex/demo.scad -o output.stl
   ```

## Code Style

We follow Rust best practices and conventions:

- **Rust Edition**: 2021
- **Formatting**: Use `rustfmt` (run `cargo fmt --all`)
- **Linting**: Use `clippy` (run `cargo clippy --all-targets --all-features -- -D warnings`)
- **Error Handling**: Use `Result` types; avoid panics in library code
- **Documentation**: Add doc comments for public APIs

### Before Submitting

Run these commands to ensure your code meets our standards:

```bash
# Format code
cargo fmt --all

# Check for issues
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test

# Run benchmarks (optional)
cargo bench
```

## Making Changes

### Branching Strategy

- `main` branch is the stable branch
- Create feature branches from `main`
- Use descriptive branch names: `feature/add-polyhedron`, `fix/stl-export-bug`

### Commit Messages

- Use clear, descriptive commit messages
- Start with a verb in present tense: "Add", "Fix", "Update", "Remove"
- Reference issues when applicable: "Fix #123: Resolve STL export bug"

### Pull Requests

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass and code is formatted
6. Submit a pull request

**PR Checklist:**
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] All tests pass (`cargo test`)
- [ ] New features have tests
- [ ] Documentation is updated (if applicable)
- [ ] SPDX headers are present in new files

## Testing

### Unit Tests

Write unit tests for new functionality:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cube_generation() {
        // Test implementation
    }
}
```

### Integration Tests

Add integration tests in the `tests/` directory for end-to-end scenarios.

### Equivalence Tests

If adding new OpenSCAD-compatible features, add equivalence tests to verify output matches OpenSCAD.

## Documentation

- Add doc comments (`///`) for public functions, structs, and modules
- Update the README.md if adding new features
- Add examples to demonstrate usage

## Licensing and DCO

By opening a pull request, you certify that:

1. You have the right to submit the work under the Apache License 2.0
2. You understand and agree that your contributions will be licensed under Apache-2.0
3. You have added the appropriate SPDX license header to any new files:
   ```rust
   // SPDX-License-Identifier: Apache-2.0
   // Copyright (c) 2025 Polyframe Inc.
   ```

This project follows the [Developer Certificate of Origin (DCO)](https://developercertificate.org/) process.

## Code of Conduct

Please note that this project has a [Code of Conduct](./CODE_OF_CONDUCT.md). By participating in this project, you agree to abide by its terms.

## Getting Help

- **Issues**: Open a GitHub issue for bugs or feature requests
- **Discussions**: Use GitHub Discussions for questions and ideas
- **Documentation**: Check [docs.rs/polyframe](https://docs.rs/polyframe)

## Project Structure

```
polyframe-kernel/
├── src/
│   ├── ast/           # Abstract Syntax Tree evaluation
│   ├── geometry/      # Primitives, meshes, boolean operations
│   ├── io/            # Parser, importers, exporters
│   ├── ffi/           # WASM and Node.js bindings
│   ├── cli/           # CLI implementation
│   └── utils/         # Math and utility functions
├── tests/             # Integration tests
├── benches/           # Performance benchmarks
└── examples/          # Sample .scad files
```

## Release Process

Releases are managed by maintainers:

1. Version is bumped in `Cargo.toml`
2. Changelog is updated
3. Git tag is created (e.g., `v0.2.0`)
4. GitHub release is created automatically
5. Crate is published to crates.io

---

Thank you for contributing to Polyframe Kernel! 🦀

