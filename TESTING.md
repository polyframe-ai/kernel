# Testing the Polyframe Kernel

## Quick Test Commands

### Run All Tests
```bash
cargo test
```

### Run Only Parser Tests (Fast)
```bash
cargo test --lib io::parser::tests
```

### Run Specific Test
```bash
cargo test test_parse_variable_assignment
```

### Run Tests with Output
```bash
cargo test -- --nocapture
```

### Run Tests for Variable Assignments Only
```bash
cargo test variable_assignment
```

## Development Workflow

### 1. Make Changes to Parser
Edit `src/io/parser.rs` or `src/io/scad.pest`

### 2. Run Tests Immediately
```bash
cargo test --lib io::parser::tests
```

### 3. Test Specific Failing Case
```bash
cargo test test_parse_parametric_box_example -- --nocapture
```

### 4. Run All Parser Tests
```bash
cargo test --lib io
```

## Test Structure

### Unit Tests (in `src/io/parser.rs`)
- `test_parse_cube` - Basic cube parsing
- `test_parse_transform` - Transform operations
- `test_parse_boolean` - Boolean operations
- `test_parse_variable_assignment` - Single variable assignment
- `test_parse_multiple_variable_assignments` - Multiple variables
- `test_parse_variable_assignment_with_geometry` - Variables + geometry
- `test_parse_parametric_box_example` - Real-world example
- `test_parse_variable_with_number` - Number assignment
- `test_parse_variable_with_vector` - Vector assignment
- `test_parse_variable_with_expression` - Expression assignment

## Testing Without Full Build

You don't need to build the full executable to test parser changes:

1. **Parser tests only** (fastest):
   ```bash
   cargo test --lib io::parser::tests
   ```

2. **All IO tests**:
   ```bash
   cargo test --lib io
   ```

3. **All library tests** (no binary):
   ```bash
   cargo test --lib
   ```

## Debugging Failed Tests

### See Detailed Error Messages
```bash
cargo test -- --nocapture
```

### Run Single Test with Output
```bash
cargo test test_parse_variable_assignment -- --nocapture
```

### Check Parser Grammar
The grammar file is at `src/io/scad.pest`. After changes, run:
```bash
cargo test --lib io::parser::tests
```

## Common Issues

### Test Fails with Parse Error
- Check `src/io/scad.pest` grammar rules
- Verify the parser handles the new rule in `parse_statement()`
- Run with `--nocapture` to see the exact error

### Tests Pass but App Still Fails
- Make sure you rebuild the kernel: `cargo build --release`
- Check that the app is using the rebuilt kernel
- Verify the kernel path in the app

## Integration with App

After tests pass, rebuild for the app:
```bash
# Build release version
cargo build --release

# The executable will be at:
# target/release/polyframe
```

Then update the app to use the new kernel executable.

