# Third-Party Licenses

This project uses the following Rust crates (see Cargo.lock for exact versions):

## Parser & Language
- **pest** — MIT / Apache-2.0
- **pest_derive** — MIT / Apache-2.0

## Math and Geometry
- **nalgebra** — Apache-2.0
- **parry3d** — Apache-2.0

## Serialization
- **serde** — MIT / Apache-2.0
- **serde_json** — MIT / Apache-2.0

## I/O
- **stl_io** — MIT / Apache-2.0
- **thiserror** — MIT / Apache-2.0
- **anyhow** — MIT / Apache-2.0

## FFI/WASM
- **wasm-bindgen** — MIT / Apache-2.0
- **napi** — MIT
- **napi-derive** — MIT

## CLI
- **clap** — MIT / Apache-2.0
- **colored** — MPL-2.0
- **indicatif** — MIT

## Utilities
- **rayon** — MIT / Apache-2.0
- **ahash** — MIT / Apache-2.0
- **dashmap** — MIT
- **tempfile** — MIT / Apache-2.0

## Development Dependencies
- **criterion** — MIT / Apache-2.0
- **approx** — Apache-2.0

## License Compatibility

All dependencies are licensed under permissive open source licenses (MIT, Apache-2.0, MPL-2.0) that are compatible with the Apache-2.0 license of this project.

Each crate's full license text is included in its repository and on its crates.io page.

## Verification

To verify the licenses of all dependencies, run:

```bash
cargo install cargo-license
cargo license
```

For detailed license information, visit:
- crates.io for each package
- The respective GitHub repositories linked on crates.io

