// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Grammar-based fuzz testing for SCAD generation
//! Generates random valid SCAD code to test parser and renderer

use anyhow::Result;
use rand::Rng;

/// Fuzzer configuration
pub struct FuzzerConfig {
    pub count: usize,
    pub max_depth: usize,
    pub max_primitives: usize,
}

impl Default for FuzzerConfig {
    fn default() -> Self {
        Self {
            count: 500,
            max_depth: 5,
            max_primitives: 10,
        }
    }
}

/// Fuzzer for generating random SCAD code
pub struct Fuzzer {
    config: FuzzerConfig,
    rng: rand::rngs::ThreadRng,
}

impl Fuzzer {
    pub fn new(config: FuzzerConfig) -> Self {
        Self {
            config,
            rng: rand::thread_rng(),
        }
    }

    /// Generate a random SCAD program
    pub fn generate(&mut self, depth: usize) -> String {
        if depth >= self.config.max_depth {
            return self.generate_primitive();
        }

        match self.rng.gen_range(0..=4) {
            0 => self.generate_primitive(),
            1 => self.generate_transform(depth),
            2 => self.generate_boolean(depth),
            3 => self.generate_module(depth),
            _ => self.generate_primitive(),
        }
    }

    /// Generate a random primitive
    fn generate_primitive(&mut self) -> String {
        match self.rng.gen_range(0..=2) {
            0 => self.generate_cube(),
            1 => self.generate_sphere(),
            2 => self.generate_cylinder(),
            _ => self.generate_cube(),
        }
    }

    /// Generate a random cube
    fn generate_cube(&mut self) -> String {
        let size = self.rng.gen_range(1.0..=50.0);
        let center = self.rng.gen_bool(0.3);
        
        if self.rng.gen_bool(0.5) {
            format!("cube([{:.2}, {:.2}, {:.2}], center={});", size, size * 1.5, size * 0.8, center)
        } else {
            format!("cube({:.2}, center={});", size, center)
        }
    }

    /// Generate a random sphere
    fn generate_sphere(&mut self) -> String {
        let r = self.rng.gen_range(1.0..=30.0);
        let fn_val = if self.rng.gen_bool(0.3) {
            format!(", $fn={}", self.rng.gen_range(8..=64))
        } else {
            String::new()
        };
        format!("sphere(r={:.2}{});", r, fn_val)
    }

    /// Generate a random cylinder
    fn generate_cylinder(&mut self) -> String {
        let h = self.rng.gen_range(1.0..=50.0);
        let r1 = self.rng.gen_range(1.0..=20.0);
        let r2 = if self.rng.gen_bool(0.3) {
            self.rng.gen_range(1.0..=20.0)
        } else {
            r1
        };
        let center = self.rng.gen_bool(0.3);
        let fn_val = if self.rng.gen_bool(0.3) {
            format!(", $fn={}", self.rng.gen_range(8..=64))
        } else {
            String::new()
        };
        format!("cylinder(h={:.2}, r1={:.2}, r2={:.2}, center={}{});", h, r1, r2, center, fn_val)
    }

    /// Generate a random transform
    fn generate_transform(&mut self, depth: usize) -> String {
        let child = self.generate(depth + 1);
        match self.rng.gen_range(0..=3) {
            0 => self.generate_translate(child),
            1 => self.generate_rotate(child),
            2 => self.generate_scale(child),
            3 => self.generate_mirror(child),
            _ => self.generate_translate(child),
        }
    }

    /// Generate translate transform
    fn generate_translate(&mut self, child: String) -> String {
        let x = self.rng.gen_range(-50.0..=50.0);
        let y = self.rng.gen_range(-50.0..=50.0);
        let z = self.rng.gen_range(-50.0..=50.0);
        format!("translate([{:.2}, {:.2}, {:.2}]) {{\n    {}\n}}", x, y, z, child)
    }

    /// Generate rotate transform
    fn generate_rotate(&mut self, child: String) -> String {
        let x = self.rng.gen_range(0.0..=360.0);
        let y = self.rng.gen_range(0.0..=360.0);
        let z = self.rng.gen_range(0.0..=360.0);
        format!("rotate([{:.2}, {:.2}, {:.2}]) {{\n    {}\n}}", x, y, z, child)
    }

    /// Generate scale transform
    fn generate_scale(&mut self, child: String) -> String {
        if self.rng.gen_bool(0.5) {
            let s = self.rng.gen_range(0.1..=3.0);
            format!("scale({:.2}) {{\n    {}\n}}", s, child)
        } else {
            let x = self.rng.gen_range(0.1..=3.0);
            let y = self.rng.gen_range(0.1..=3.0);
            let z = self.rng.gen_range(0.1..=3.0);
            format!("scale([{:.2}, {:.2}, {:.2}]) {{\n    {}\n}}", x, y, z, child)
        }
    }

    /// Generate mirror transform
    fn generate_mirror(&mut self, child: String) -> String {
        let x = if self.rng.gen_bool(0.5) { 1.0 } else { 0.0 };
        let y = if self.rng.gen_bool(0.5) { 1.0 } else { 0.0 };
        let z = if self.rng.gen_bool(0.5) { 1.0 } else { 0.0 };
        format!("mirror([{:.1}, {:.1}, {:.1}]) {{\n    {}\n}}", x, y, z, child)
    }

    /// Generate a random boolean operation
    fn generate_boolean(&mut self, depth: usize) -> String {
        let count = self.rng.gen_range(2..=self.config.max_primitives.min(5));
        let mut children = Vec::new();
        for _ in 0..count {
            children.push(self.generate(depth + 1));
        }
        
        match self.rng.gen_range(0..=2) {
            0 => format!("union() {{\n    {}\n}}", children.join("\n    ")),
            1 => format!("difference() {{\n    {}\n    {}\n}}", children[0], children[1]),
            2 => format!("intersection() {{\n    {}\n}}", children.join("\n    ")),
            _ => format!("union() {{\n    {}\n}}", children.join("\n    ")),
        }
    }

    /// Generate a random module (simplified - just a wrapper)
    fn generate_module(&mut self, depth: usize) -> String {
        let name = format!("module_{}", self.rng.gen_range(0..=1000));
        let body = self.generate(depth + 1);
        format!("module {}() {{\n    {}\n}}\n\n{}();", name, body, name)
    }

    /// Run fuzz testing and return generated SCAD strings
    pub fn run(&mut self) -> Vec<(String, String)> {
        let mut results = Vec::new();
        
        for i in 0..self.config.count {
            let scad_code = self.generate(0);
            let name = format!("fuzz_{:05}", i);
            results.push((name, scad_code));
        }
        
        results
    }
}

/// Test parse success/failure parity between OpenSCAD and Polyframe
pub fn test_parse_parity(scad_code: &str) -> Result<(bool, bool)> {
    // Try to parse with Polyframe
    let polyframe_parse = crate::parse_scad(scad_code).is_ok();
    
    // Try to parse with OpenSCAD (if available)
    // This is a simplified check - in practice we'd call OpenSCAD
    let openscad_parse = true; // Assume success for now
    
    Ok((polyframe_parse, openscad_parse))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzer_generation() {
        let config = FuzzerConfig {
            count: 10,
            max_depth: 3,
            max_primitives: 5,
        };
        let mut fuzzer = Fuzzer::new(config);
        let results = fuzzer.run();
        
        assert_eq!(results.len(), 10);
        for (name, code) in results {
            assert!(!code.is_empty());
            assert!(!name.is_empty());
        }
    }

    #[test]
    fn test_generate_primitive() {
        let config = FuzzerConfig::default();
        let mut fuzzer = Fuzzer::new(config);
        
        let cube = fuzzer.generate_cube();
        assert!(cube.contains("cube"));
        
        let sphere = fuzzer.generate_sphere();
        assert!(sphere.contains("sphere"));
        
        let cylinder = fuzzer.generate_cylinder();
        assert!(cylinder.contains("cylinder"));
    }
}

