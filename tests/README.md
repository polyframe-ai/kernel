# Tests

## Directory Structure

### `integration/`
Integration tests that verify end-to-end functionality:
- `io_equivalence.rs` - Verify Polyframe outputs match OpenSCAD
- `evaluation.rs` - Evaluation harness tests

### `fixtures/`
Test data and exercise datasets:
- `polyframe_exercises_001_040.json` - Exercise set 1-40
- `polyframe_exercises_041_100.json` - Exercise set 41-100

## Running Tests

### All tests
```bash
cargo test
```

### Integration tests only
```bash
cargo test --test '*'
```

### Specific test
```bash
cargo test --test io_equivalence
```

## Test Fixtures

The JSON exercise files contain parametric test cases for evaluating Polyframe's compatibility with OpenSCAD. Each exercise defines:
- Input `.scad` code
- Expected behavior
- Tolerance thresholds

