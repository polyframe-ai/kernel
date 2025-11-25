// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Performance benchmarks

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use nalgebra::Vector3;
use polyframe::{ast::*, geometry::Primitive, render, Kernel};

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");

    let simple = "cube([10, 10, 10]);";
    group.bench_with_input(BenchmarkId::new("simple_cube", ""), &simple, |b, source| {
        b.iter(|| polyframe::io::parse_scad(black_box(source)).unwrap());
    });

    let complex = r#"
        difference() {
            cube([20, 20, 20]);
            translate([10, 10, 10])
                sphere(r=15);
        }
    "#;
    group.bench_with_input(BenchmarkId::new("complex", ""), &complex, |b, source| {
        b.iter(|| polyframe::io::parse_scad(black_box(source)).unwrap());
    });

    group.finish();
}

fn bench_primitives(c: &mut Criterion) {
    let mut group = c.benchmark_group("primitives");

    group.bench_function("cube", |b| {
        b.iter(|| Primitive::cube(black_box(Vector3::new(10.0, 10.0, 10.0)), true).to_mesh());
    });

    group.bench_function("sphere_32", |b| {
        b.iter(|| Primitive::sphere(black_box(10.0), black_box(32)).to_mesh());
    });

    group.bench_function("sphere_64", |b| {
        b.iter(|| Primitive::sphere(black_box(10.0), black_box(64)).to_mesh());
    });

    group.bench_function("cylinder", |b| {
        b.iter(|| Primitive::cylinder(black_box(20.0), black_box(5.0), black_box(32)).to_mesh());
    });

    group.finish();
}

fn bench_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("render");

    group.bench_function("cube", |b| {
        b.iter(|| render(black_box("cube([10, 10, 10]);")).unwrap());
    });

    group.bench_function("sphere", |b| {
        b.iter(|| render(black_box("sphere(r=10);")).unwrap());
    });

    group.bench_function("transform", |b| {
        b.iter(|| {
            render(black_box(
                "translate([5, 0, 0]) rotate([0, 45, 0]) cube([10, 10, 10]);",
            ))
            .unwrap()
        });
    });

    group.bench_function("union", |b| {
        b.iter(|| {
            render(black_box(
                "union() { cube(10); translate([8, 0, 0]) cube(10); }",
            ))
            .unwrap()
        });
    });

    group.finish();
}

fn bench_boolean_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("boolean_ops");

    let cube1 = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), true).to_mesh();
    let cube2 = Primitive::cube(Vector3::new(8.0, 8.0, 8.0), true).to_mesh();

    group.bench_function("union", |b| {
        b.iter(|| {
            cube1
                .boolean_operation(
                    black_box(&cube2),
                    black_box(polyframe::geometry::BooleanOp::Union),
                )
                .unwrap()
        });
    });

    group.bench_function("difference", |b| {
        b.iter(|| {
            cube1
                .boolean_operation(
                    black_box(&cube2),
                    black_box(polyframe::geometry::BooleanOp::Difference),
                )
                .unwrap()
        });
    });

    group.finish();
}

fn bench_incremental_vs_full(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_vs_full");

    // Create a complex AST with multiple nodes
    let child1 = Node::with_id(
        NodeKind::Cube {
            size: Vector3::new(10.0, 10.0, 10.0),
            center: false,
        },
        "cube1".into(),
    );
    let child2 = Node::with_id(NodeKind::Sphere { r: 5.0, fn_: 32 }, "sphere1".into());
    let child3 = Node::with_id(
        NodeKind::Cylinder {
            h: 20.0,
            r: 3.0,
            fn_: 32,
        },
        "cylinder1".into(),
    );

    let union_node = Node::with_id(
        NodeKind::Union(vec![child1.clone(), child2.clone(), child3.clone()]),
        "union1".into(),
    );

    // Full evaluation (standard)
    group.bench_function("full_evaluation", |b| {
        b.iter(|| {
            let evaluator = Evaluator::new();
            evaluator.evaluate(black_box(&union_node)).unwrap()
        });
    });

    // Incremental evaluation (first run)
    group.bench_function("incremental_first_run", |b| {
        b.iter(|| {
            let evaluator = IncrementalEvaluator::from_ast(black_box(&union_node));
            evaluator.evaluate(black_box(&union_node)).unwrap()
        });
    });

    // Incremental evaluation (cached)
    group.bench_function("incremental_cached", |b| {
        let evaluator = IncrementalEvaluator::from_ast(&union_node);
        // Pre-populate cache
        let _ = evaluator.evaluate(&union_node);

        b.iter(|| evaluator.evaluate(black_box(&union_node)).unwrap());
    });

    // Kernel update subtree
    group.bench_function("kernel_update_subtree", |b| {
        let mut kernel = Kernel::with_ast(union_node.clone());
        let _ = kernel.render();

        let updated_child = Node::with_id(NodeKind::Sphere { r: 8.0, fn_: 32 }, "sphere1".into());

        b.iter(|| {
            kernel
                .update_subtree(
                    black_box(&"sphere1".to_string()),
                    black_box(updated_child.clone()),
                )
                .unwrap()
        });
    });

    group.finish();
}

fn bench_parallel_vs_sequential(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_vs_sequential");

    // Create AST with multiple children for parallel processing
    let children: Vec<Node> = (0..8)
        .map(|i| {
            Node::with_id(
                NodeKind::Sphere { r: 5.0, fn_: 32 },
                format!("sphere_{}", i),
            )
        })
        .collect();

    let union_node = Node::with_id(NodeKind::Union(children.clone()), "union_parallel".into());

    // Sequential evaluation
    group.bench_function("sequential", |b| {
        b.iter(|| {
            let evaluator = Evaluator::new();
            evaluator.evaluate(black_box(&union_node)).unwrap()
        });
    });

    // Parallel evaluation
    group.bench_function("parallel", |b| {
        b.iter(|| ParallelEvaluator::evaluate(black_box(&union_node)).unwrap());
    });

    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.sample_size(10); // Fewer samples for memory tests

    // Large mesh generation
    let large_union: Vec<Node> = (0..50)
        .map(|i| {
            Node::new(NodeKind::Cube {
                size: Vector3::new(5.0 + i as f32 * 0.1, 5.0, 5.0),
                center: false,
            })
        })
        .collect();

    let union_node = Node::new(NodeKind::Union(large_union));

    group.bench_function("large_mesh_generation", |b| {
        b.iter(|| {
            let evaluator = Evaluator::new();
            let mesh = evaluator.evaluate(black_box(&union_node)).unwrap();
            assert!(mesh.vertex_count() > 0);
        });
    });

    group.finish();
}

fn bench_cache_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_effectiveness");

    // Build a tree with repeated nodes
    let base_cube = Node::with_id(
        NodeKind::Cube {
            size: Vector3::new(10.0, 10.0, 10.0),
            center: false,
        },
        "base_cube".into(),
    );

    let transform1 = Node::with_id(
        NodeKind::Transform {
            op: TransformOp::Translate(Vector3::new(10.0, 0.0, 0.0)),
            children: vec![base_cube.clone()],
        },
        "transform1".into(),
    );

    let transform2 = Node::with_id(
        NodeKind::Transform {
            op: TransformOp::Translate(Vector3::new(0.0, 10.0, 0.0)),
            children: vec![base_cube.clone()],
        },
        "transform2".into(),
    );

    let union = Node::with_id(
        NodeKind::Union(vec![transform1, transform2]),
        "union_cached".into(),
    );

    group.bench_function("with_cache", |b| {
        b.iter(|| {
            let evaluator = IncrementalEvaluator::from_ast(black_box(&union));
            evaluator.evaluate(black_box(&union)).unwrap()
        });
    });

    group.bench_function("without_cache", |b| {
        b.iter(|| {
            let evaluator = Evaluator::new();
            evaluator.evaluate(black_box(&union)).unwrap()
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parse,
    bench_primitives,
    bench_render,
    bench_boolean_ops,
    bench_incremental_vs_full,
    bench_parallel_vs_sequential,
    bench_memory_usage,
    bench_cache_effectiveness
);
criterion_main!(benches);
