// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Performance benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use polyframe::{render, geometry::Primitive};
use nalgebra::Vector3;

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");
    
    let simple = "cube([10, 10, 10]);";
    group.bench_with_input(BenchmarkId::new("simple_cube", ""), &simple, |b, source| {
        b.iter(|| {
            polyframe::io::parse_scad(black_box(source)).unwrap()
        });
    });
    
    let complex = r#"
        difference() {
            cube([20, 20, 20]);
            translate([10, 10, 10])
                sphere(r=15);
        }
    "#;
    group.bench_with_input(BenchmarkId::new("complex", ""), &complex, |b, source| {
        b.iter(|| {
            polyframe::io::parse_scad(black_box(source)).unwrap()
        });
    });
    
    group.finish();
}

fn bench_primitives(c: &mut Criterion) {
    let mut group = c.benchmark_group("primitives");
    
    group.bench_function("cube", |b| {
        b.iter(|| {
            Primitive::cube(black_box(Vector3::new(10.0, 10.0, 10.0))).to_mesh()
        });
    });
    
    group.bench_function("sphere_32", |b| {
        b.iter(|| {
            Primitive::sphere(black_box(10.0), black_box(32)).to_mesh()
        });
    });
    
    group.bench_function("sphere_64", |b| {
        b.iter(|| {
            Primitive::sphere(black_box(10.0), black_box(64)).to_mesh()
        });
    });
    
    group.bench_function("cylinder", |b| {
        b.iter(|| {
            Primitive::cylinder(black_box(20.0), black_box(5.0), black_box(32)).to_mesh()
        });
    });
    
    group.finish();
}

fn bench_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("render");
    
    group.bench_function("cube", |b| {
        b.iter(|| {
            render(black_box("cube([10, 10, 10]);")).unwrap()
        });
    });
    
    group.bench_function("sphere", |b| {
        b.iter(|| {
            render(black_box("sphere(r=10);")).unwrap()
        });
    });
    
    group.bench_function("transform", |b| {
        b.iter(|| {
            render(black_box("translate([5, 0, 0]) rotate([0, 45, 0]) cube([10, 10, 10]);")).unwrap()
        });
    });
    
    group.bench_function("union", |b| {
        b.iter(|| {
            render(black_box("union() { cube(10); translate([8, 0, 0]) cube(10); }")).unwrap()
        });
    });
    
    group.finish();
}

fn bench_boolean_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("boolean_ops");
    
    let cube1 = Primitive::cube(Vector3::new(10.0, 10.0, 10.0)).to_mesh();
    let cube2 = Primitive::cube(Vector3::new(8.0, 8.0, 8.0)).to_mesh();
    
    group.bench_function("union", |b| {
        b.iter(|| {
            cube1.boolean_operation(
                black_box(&cube2),
                black_box(polyframe::geometry::BooleanOp::Union)
            ).unwrap()
        });
    });
    
    group.bench_function("difference", |b| {
        b.iter(|| {
            cube1.boolean_operation(
                black_box(&cube2),
                black_box(polyframe::geometry::BooleanOp::Difference)
            ).unwrap()
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_parse,
    bench_primitives,
    bench_render,
    bench_boolean_ops
);
criterion_main!(benches);

