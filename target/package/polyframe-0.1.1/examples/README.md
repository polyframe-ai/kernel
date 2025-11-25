# Example .scad Files

This directory contains example OpenSCAD files organized by complexity and purpose.

## Directory Structure

### `primitives/`
Basic geometric primitives:
- `cube.scad` - Simple cube primitive
- `sphere.scad` - Sphere with custom segments
- `cylinder.scad` - Cylinder primitive

### `operations/`
Boolean operations and combinations:
- `union.scad` - Union of multiple shapes
- `difference.scad` - Boolean difference operation

### `complex/`
Complex models with multiple operations:
- `demo.scad` - Complex shape with multiple operations
- `transform.scad` - Transformation operations (translate, rotate, scale)

## Usage

Run any example with:

```bash
polyframe examples/primitives/cube.scad -o output.stl
```

Or compare with OpenSCAD:

```bash
polyframe compare examples/primitives/cube.scad
```

