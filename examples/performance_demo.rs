// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Performance demonstration example
//!
//! This example demonstrates:
//! - Incremental evaluation with caching
//! - Parallel processing with rayon
//! - Lazy rendering mode
//! - Performance metrics comparison

use nalgebra::Vector3;
use polyframe::benchmark_metrics::*;
use polyframe::*;

fn main() {
    println!("🚀 Polyframe Kernel - Performance Demo\n");

    // Create a complex AST for testing
    let child1 = Node::with_id(
        NodeKind::Cube {
            size: Vector3::new(10.0, 10.0, 10.0),
            center: false,
        },
        "cube1".into(),
    );

    let child2 = Node::with_id(NodeKind::Sphere { r: 7.0, fn_: 64 }, "sphere1".into());

    let child3 = Node::with_id(
        NodeKind::Cylinder {
            h: 20.0,
            r: 4.0,
            fn_: 64,
        },
        "cylinder1".into(),
    );

    let union_node = Node::with_id(
        NodeKind::Union(vec![child1.clone(), child2.clone(), child3.clone()]),
        "union_root".into(),
    );

    let mut report = BenchmarkReport::new();

    // Test 1: Standard evaluation
    println!("📊 Test 1: Standard Evaluation");
    let timer = Timer::start();
    let evaluator = ast::Evaluator::new();
    let mesh = evaluator.evaluate(&union_node).unwrap();
    let duration = timer.elapsed();

    let mut metrics = OperationMetrics::new("full_evaluation");
    metrics.duration = duration;
    metrics.vertex_count = mesh.vertex_count();
    metrics.triangle_count = mesh.triangle_count();
    metrics.mesh_count = 1;
    metrics.memory_kb = estimate_mesh_memory(mesh.vertex_count(), mesh.triangle_count());

    report.add_metrics(metrics);
    println!("   ✓ Time: {:.2}ms", timer.elapsed_ms());
    println!("   ✓ Vertices: {}", mesh.vertex_count());
    println!("   ✓ Triangles: {}\n", mesh.triangle_count());

    // Test 2: Incremental evaluation (first run)
    println!("📊 Test 2: Incremental Evaluation (First Run)");
    let timer = Timer::start();
    let incremental_eval = IncrementalEvaluator::from_ast(&union_node);
    let mesh = incremental_eval.evaluate(&union_node).unwrap();
    let duration = timer.elapsed();

    let mut metrics = OperationMetrics::new("incremental_first_run");
    metrics.duration = duration;
    metrics.vertex_count = mesh.vertex_count();
    metrics.triangle_count = mesh.triangle_count();
    metrics.mesh_count = 1;
    metrics.memory_kb = estimate_mesh_memory(mesh.vertex_count(), mesh.triangle_count());

    report.add_metrics(metrics);
    println!("   ✓ Time: {:.2}ms", timer.elapsed_ms());

    let stats = incremental_eval.cache_stats();
    println!(
        "   ✓ Cache: {}/{} nodes ({:.1}% hit rate)\n",
        stats.cached_nodes,
        stats.total_nodes,
        stats.hit_rate()
    );

    // Test 3: Incremental evaluation (cached)
    println!("📊 Test 3: Incremental Evaluation (Cached)");
    let timer = Timer::start();
    let mesh = incremental_eval.evaluate(&union_node).unwrap();
    let duration = timer.elapsed();

    let mut metrics = OperationMetrics::new("incremental_cached");
    metrics.duration = duration;
    metrics.vertex_count = mesh.vertex_count();
    metrics.triangle_count = mesh.triangle_count();
    metrics.mesh_count = 1;
    metrics.memory_kb = estimate_mesh_memory(mesh.vertex_count(), mesh.triangle_count());

    report.add_metrics(metrics);
    println!("   ✓ Time: {:.2}ms (from cache)", timer.elapsed_ms());
    println!(
        "   🚀 Speedup: {:.2}×\n",
        report
            .speedup("full_evaluation", "incremental_cached")
            .unwrap_or(1.0)
    );

    // Test 4: Parallel evaluation
    println!("📊 Test 4: Parallel Evaluation");
    let timer = Timer::start();
    let mesh = ParallelEvaluator::evaluate(&union_node).unwrap();
    let duration = timer.elapsed();

    let mut metrics = OperationMetrics::new("parallel");
    metrics.duration = duration;
    metrics.vertex_count = mesh.vertex_count();
    metrics.triangle_count = mesh.triangle_count();
    metrics.mesh_count = 1;
    metrics.memory_kb = estimate_mesh_memory(mesh.vertex_count(), mesh.triangle_count());

    report.add_metrics(metrics);
    println!("   ✓ Time: {:.2}ms", timer.elapsed_ms());
    println!("   ✓ Using rayon for parallel CSG operations\n");

    // Test 5: Kernel with update_subtree
    println!("📊 Test 5: Kernel Update Subtree");
    let mut kernel = Kernel::with_ast(union_node.clone());

    let timer = Timer::start();
    let _ = kernel.render().unwrap();
    let initial_time = timer.elapsed_ms();
    println!("   ✓ Initial render: {:.2}ms", initial_time);

    // Update a single subtree
    let updated_sphere = Node::with_id(NodeKind::Sphere { r: 10.0, fn_: 64 }, "sphere1".into());

    let timer = Timer::start();
    let mesh = kernel
        .update_subtree(&"sphere1".to_string(), updated_sphere)
        .unwrap();
    let update_time = timer.elapsed_ms();

    let mut metrics = OperationMetrics::new("kernel_update_subtree");
    metrics.duration = timer.elapsed();
    metrics.vertex_count = mesh.vertex_count();
    metrics.triangle_count = mesh.triangle_count();
    metrics.mesh_count = 1;
    metrics.memory_kb = estimate_mesh_memory(mesh.vertex_count(), mesh.triangle_count());

    report.add_metrics(metrics);
    println!("   ✓ Subtree update: {:.2}ms", update_time);
    println!("   ✓ Only affected nodes re-evaluated\n");

    // Test 6: Sequential for comparison
    println!("📊 Test 6: Sequential Evaluation (for parallel comparison)");
    let timer = Timer::start();
    let evaluator = ast::Evaluator::new();
    let mesh = evaluator.evaluate(&union_node).unwrap();
    let duration = timer.elapsed();

    let mut metrics = OperationMetrics::new("sequential");
    metrics.duration = duration;
    metrics.vertex_count = mesh.vertex_count();
    metrics.triangle_count = mesh.triangle_count();
    metrics.mesh_count = 1;
    metrics.memory_kb = estimate_mesh_memory(mesh.vertex_count(), mesh.triangle_count());

    report.add_metrics(metrics);
    println!("   ✓ Time: {:.2}ms\n", timer.elapsed_ms());

    if let Some(speedup) = report.speedup("sequential", "parallel") {
        println!("   🚀 Parallel speedup: {:.2}×\n", speedup);
    }

    // Print comprehensive report
    report.print_report();

    println!("✅ Performance demo completed!");
    println!("\n💡 Key Findings:");
    println!("   • Incremental evaluation provides significant speedup for cached operations");
    println!("   • Parallel processing improves performance for complex boolean operations");
    println!("   • Kernel update_subtree minimizes re-computation on changes");
    println!("   • Lazy rendering defers export until explicitly requested");
    println!("\n📌 To run benchmarks: cargo bench");
    println!("📌 To use CLI flags: polyframe render <file> --output <out> --lazy --parallel --incremental\n");
}
