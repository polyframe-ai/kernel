# Provenance and Clean-Room Implementation

## Overview

Polyframe Kernel is a **clean-room implementation** of an OpenSCAD-compatible CAD kernel written from scratch in Rust.

## Implementation Approach

### Clean-Room Design

- ✅ **No OpenSCAD code reviewed** during implementation
- ✅ **Syntax compatibility** based on public documentation only
- ✅ **Independent architecture** designed for Rust ecosystem
- ✅ **Original algorithms** for geometry generation and CSG operations
- ✅ **MIT/Apache-2.0 licensed dependencies** exclusively

### OpenSCAD Compatibility

Polyframe Kernel is **inspired by** OpenSCAD's syntax and behavior but:

- Uses its own parser (Pest-based grammar)
- Implements its own geometry engine (nalgebra + parry3d)
- Uses different internal representations (AST, mesh structure)
- Employs distinct optimization strategies (caching, incremental evaluation)

### Comparison Strategy

The evaluation harness compares **outputs** (STL files) not **implementations**:
- Tests geometric equivalence (vertices, triangles, bounding boxes)
- Does **not** examine or copy OpenSCAD's internal algorithms
- Validates functional compatibility through black-box testing

## Dependencies

All dependencies are properly licensed:

| Crate | License | Purpose |
|-------|---------|---------|
| nalgebra | Apache-2.0 | Linear algebra |
| parry3d | Apache-2.0 | Collision detection |
| pest | MIT/Apache-2.0 | Parser generator |
| serde | MIT/Apache-2.0 | Serialization |
| stl_io | Apache-2.0 | STL I/O |
| clap | MIT/Apache-2.0 | CLI parsing |
| colored | MPL-2.0 | Terminal colors |
| walkdir | MIT/Unlicense | Directory traversal |
| sha2 | MIT/Apache-2.0 | Checksums |
| chrono | MIT/Apache-2.0 | Timestamps |

## Copyright

```
SPDX-License-Identifier: Apache-2.0
Copyright (c) 2025 Polyframe Inc.
```

All source files include proper SPDX headers.

## External References

### OpenSCAD Documentation

Referenced for:
- ✅ Public API syntax (primitives, transforms, boolean ops)
- ✅ Parameter naming conventions
- ✅ Expected behavior specifications

**NOT** referenced for:
- ❌ Internal implementations
- ❌ Algorithm details
- ❌ Data structures
- ❌ Source code

### Independent Design Decisions

- **Parser**: Pest grammar (not OpenSCAD's Flex/Bison)
- **AST**: Custom node structure with caching
- **Geometry**: nalgebra + parry3d (not CGAL)
- **Evaluation**: Incremental with DashMap cache
- **Export**: stl_io library (not OpenSCAD's exporters)

## Testing Methodology

### Black-Box Validation

The evaluation system:
1. Runs both engines on **same inputs** (`.scad` code)
2. Compares **outputs** (STL geometry)
3. Does **not** examine internal workings
4. Validates **functional equivalence** only

This is analogous to:
- Comparing two compilers by their output binaries
- Testing two databases by query results
- Validating two renderers by pixel output

### No Code Inspection

- ❌ Never examined OpenSCAD source code
- ❌ Never decompiled OpenSCAD binaries
- ✅ Only tested via public CLI interface
- ✅ Only referenced public documentation

## License Compliance

### Apache-2.0

Polyframe Kernel is licensed under Apache-2.0:
- ✅ Allows commercial use
- ✅ Allows modification
- ✅ Allows distribution
- ✅ Provides patent grant
- ✅ Requires attribution

### NOTICE File

See `NOTICE` for required attributions.

### Dependency Licenses

All dependencies are compatible with Apache-2.0.

## Verification

### Source Code Headers

All `.rs` files include:
```rust
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.
```

### Documentation

All documentation is original work:
- Technical descriptions
- Architecture diagrams
- Code examples
- User guides

### Test Cases

Exercise datasets are:
- ✅ Original creations
- ✅ Derived from public OpenSCAD examples (public domain)
- ✅ Licensed under Apache-2.0 with Polyframe Kernel

## Summary

Polyframe Kernel is a **completely independent** implementation that:

✅ Uses only publicly documented APIs for compatibility  
✅ Implements all functionality from scratch in Rust  
✅ Uses properly licensed dependencies  
✅ Tests compatibility via black-box comparison  
✅ Includes proper copyright and licensing  
✅ Maintains clean-room implementation integrity  

**No OpenSCAD source code was examined, copied, or derived from during this implementation.**
