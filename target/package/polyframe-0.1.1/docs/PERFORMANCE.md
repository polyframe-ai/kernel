# Polyframe Kernel - Performance Optimizations

This document describes the performance optimization features implemented in the Polyframe Kernel.

## Overview

The Polyframe Kernel now includes several performance optimizations that significantly improve rendering speed and memory efficiency:

- **Incremental Evaluation**: Cache AST subtrees and re-evaluate only changed nodes
- **Multi-threaded Processing**: Parallel CSG operations using rayon
- **Lazy Rendering**: Defer rendering until explicitly requested
- **Dependency Tracking**: Intelligent cache invalidation based on node relationships

## Features

### 1. Incremental Evaluation Engine

The incremental evaluator caches evaluated mesh data for each AST node and only re-evaluates nodes that have changed or depend on changed nodes.

**Key Components:**
- `IncrementalEvaluator`: Main evaluator with built-in caching
- `DependencyGraph`: Tracks parent-child relationships between nodes
- `MeshCache`: Thread-safe cache using `DashMap` and `Arc<RwLock<Mesh>>`

**Usage:**
```rust
use polyframe::IncrementalEvaluator;

let evaluator = IncrementalEvaluator::from_ast(&ast);
let mesh = evaluator.evaluate(&ast)?;

// Subsequent evaluations use cached results
let mesh2 = evaluator.evaluate(&ast)?; // Much faster!

// Check cache statistics
let stats = evaluator.cache_stats();
println!("Hit rate: {:.1}%", stats.hit_rate());
```

**Expected Speedup:** 4-10× for cached operations

### 2. Kernel API with Partial Rebuilds

The `Kernel` struct provides a high-level API for managing AST and triggering incremental re-evaluation.

**Usage:**
```rust
use polyframe::Kernel;

// Initialize kernel with AST
let mut kernel = Kernel::with_ast(ast);

// Initial render
let mesh = kernel.render()?;

// Update a specific subtree (only affected nodes are re-evaluated)
let updated_node = Node::with_id(
    NodeKind::Sphere { r: 15.0, fn_: 64 },
    "sphere1".into()
);

kernel.update_subtree(&"sphere1".to_string(), updated_node)?;
let mesh = kernel.render()?; // Fast incremental update!
```

**API Methods:**
- `Kernel::new()`: Create a new kernel
- `Kernel::with_ast(ast)`: Initialize with AST
- `Kernel::render()`: Full render
- `Kernel::update_subtree(node_id, updated_node)`: Incremental update
- `Kernel::invalidate(node_id)`: Manually invalidate cache
- `Kernel::cache_stats()`: Get cache statistics

### 3. Multi-threaded CSG Operations

Parallel evaluation using rayon for concurrent processing of independent nodes.

**Components:**
- `ParallelEvaluator`: Evaluates AST nodes in parallel
- `ParallelBooleanExecutor`: Parallel boolean operations
- `ThreadSafeMesh`: Thread-safe mesh wrapper

**Usage:**
```rust
use polyframe::ParallelEvaluator;

// Automatically parallelizes independent subtrees
let mesh = ParallelEvaluator::evaluate(&ast)?;
```

**Expected Speedup:** 2-4× on multi-core systems for complex models

### 4. Lazy Rendering Mode

Defer export operations until explicitly requested, keeping intermediate meshes in memory.

**CLI Usage:**
```bash
# Parse and evaluate, but skip export
polyframe render input.scad --output out.stl --lazy

# Combine with other flags
polyframe render input.scad --output out.stl --lazy --incremental
```

**Benefits:**
- Faster iteration during development
- Reduced I/O operations
- Useful for batch processing pipelines

### 5. Thread-Safe Mesh Structures

All mesh operations are thread-safe using `Arc<RwLock<Mesh>>`.

**Usage:**
```rust
use polyframe::geometry::{ThreadSafeMesh, ThreadSafeMeshOps};
use std::sync::{Arc, RwLock};

let mesh = create_mesh();
let safe_mesh: ThreadSafeMesh = Arc::new(RwLock::new(mesh));

// Thread-safe operations
let cloned = safe_mesh.clone_mesh();
safe_mesh.transform_safe(&matrix);
```

## CLI Flags

The following CLI flags enable performance features:

```bash
polyframe render <INPUT> --output <OUTPUT> [FLAGS]

Flags:
  --lazy          Lazy rendering (defer export)
  --parallel      Use parallel evaluation
  --incremental   Use incremental evaluation with caching
  -v, --verbose   Show detailed performance metrics
```

**Example:**
```bash
# Use all optimizations
polyframe render complex.scad --output out.stl \
  --incremental --parallel --verbose
```

## Benchmarks

Run the full benchmark suite:

```bash
cargo bench
```

### Benchmark Groups

1. **parse**: AST parsing performance
2. **primitives**: Primitive mesh generation
3. **render**: Full rendering pipeline
4. **boolean_ops**: CSG boolean operations
5. **incremental_vs_full**: Incremental vs full evaluation comparison
6. **parallel_vs_sequential**: Parallel vs sequential evaluation
7. **memory_usage**: Memory consumption tests
8. **cache_effectiveness**: Cache hit rate analysis

### Performance Demo

Run the interactive performance demo:

```bash
cargo run --example performance_demo
```

This demonstrates:
- Standard evaluation baseline
- Incremental evaluation (first run vs cached)
- Parallel evaluation
- Kernel subtree updates
- Sequential vs parallel comparison
- Detailed metrics report

## Performance Metrics

The benchmark harness tracks:

- **Time (ms)**: Operation duration in milliseconds
- **Memory (MB)**: Estimated memory usage in megabytes
- **Mesh Count**: Number of meshes generated
- **Vertex Count**: Total vertices in output mesh
- **Triangle Count**: Total triangles in output mesh

### Expected Results

Based on typical workloads:

| Operation | Baseline | Optimized | Speedup |
|-----------|----------|-----------|---------|
| Full Evaluation | 100ms | - | 1.0× |
| Incremental (cached) | - | 10-20ms | 5-10× |
| Parallel (8 cores) | 150ms | 50ms | 3× |
| Update Subtree | 100ms | 15-25ms | 4-6× |

*Note: Actual results vary based on model complexity and hardware.*

## Best Practices

### When to Use Incremental Evaluation

✅ **Use when:**
- Working on large, complex models
- Making iterative changes to specific parts
- Node IDs are properly assigned
- Memory is available for caching

❌ **Don't use when:**
- Model changes frequently and completely
- Memory is constrained
- Model is very simple (overhead > benefit)

### When to Use Parallel Evaluation

✅ **Use when:**
- Model has many independent subtrees
- Multi-core CPU is available
- Boolean operations dominate processing time
- Model has 8+ independent nodes

❌ **Don't use when:**
- Model is very simple
- Operations are sequential by nature
- Single-core performance is critical

### Optimal Node ID Strategy

For best incremental evaluation performance:

1. **Assign IDs to reusable components:**
   ```rust
   let wheel = Node::with_id(
       NodeKind::Cylinder { h: 2.0, r: 5.0, fn_: 32 },
       "wheel".into()
   );
   ```

2. **Use semantic naming:**
   ```rust
   "base_plate"
   "mounting_bracket_left"
   "bolt_hole_1"
   ```

3. **Don't over-assign:** Only ID nodes you'll reference or update

## Implementation Details

### Dependency Graph

The dependency graph tracks:
- **Children**: Direct child nodes
- **Parents**: Direct parent nodes
- **Descendants**: All nodes below in tree
- **Ancestors**: All nodes above in tree
- **Affected nodes**: Node + ancestors (for cache invalidation)

### Cache Invalidation

When a node is updated:
1. Find all affected nodes (node + ancestors)
2. Remove affected nodes from cache
3. Rebuild dependency graph
4. Re-evaluate (cached nodes are reused)

### Thread Safety

Thread safety is achieved through:
- `DashMap` for concurrent cache access
- `Arc<RwLock<Mesh>>` for shared mesh ownership
- Rayon's work-stealing scheduler for parallel execution

## Troubleshooting

### Cache Not Improving Performance

**Issue:** Incremental evaluation no slower/faster than standard

**Solutions:**
- Ensure nodes have assigned IDs
- Check cache hit rate with `.cache_stats()`
- Verify nodes aren't changing every evaluation

### Parallel Evaluation Slower

**Issue:** Parallel mode slower than sequential

**Solutions:**
- Check CPU core count (need 4+ for benefit)
- Ensure model has independent subtrees
- Profile to find bottlenecks
- Consider reducing parallelism granularity

### Memory Usage High

**Issue:** Application using too much memory

**Solutions:**
- Reduce cache size by limiting IDs
- Use standard evaluator for simple models
- Call `.invalidate()` to clear unneeded cache
- Monitor with `.cache_stats()`

## Future Improvements

Planned enhancements:

- [ ] LRU cache eviction for memory management
- [ ] Persistent cache across runs
- [ ] GPU-accelerated boolean operations
- [ ] Incremental export (only changed geometry)
- [ ] Automatic parallelization hints
- [ ] Cache prewarming strategies

## Contributing

To add new performance optimizations:

1. Add benchmarks in `benches/performance.rs`
2. Update this documentation
3. Ensure OpenSCAD compatibility maintained
4. Include performance regression tests

## License

Apache-2.0 - See LICENSE file for details

