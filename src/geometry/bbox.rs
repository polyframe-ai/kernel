// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Bounding box utilities

use super::Vertex;
use nalgebra::Point3;
use serde::{Deserialize, Serialize};

/// Axis-aligned bounding box
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

impl BoundingBox {
    pub fn new(min: Point3<f32>, max: Point3<f32>) -> Self {
        Self { min, max }
    }

    pub fn empty() -> Self {
        Self {
            min: Point3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Point3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }

    pub fn from_vertices(vertices: &[Vertex]) -> Self {
        if vertices.is_empty() {
            return Self::empty();
        }

        let mut bbox = Self::empty();
        for vertex in vertices {
            bbox.expand_to_include(&vertex.position);
        }
        bbox
    }

    pub fn expand_to_include(&mut self, point: &Point3<f32>) {
        self.min.x = self.min.x.min(point.x);
        self.min.y = self.min.y.min(point.y);
        self.min.z = self.min.z.min(point.z);
        
        self.max.x = self.max.x.max(point.x);
        self.max.y = self.max.y.max(point.y);
        self.max.z = self.max.z.max(point.z);
    }

    pub fn center(&self) -> Point3<f32> {
        Point3::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
            (self.min.z + self.max.z) / 2.0,
        )
    }

    pub fn size(&self) -> nalgebra::Vector3<f32> {
        nalgebra::Vector3::new(
            self.max.x - self.min.x,
            self.max.y - self.min.y,
            self.max.z - self.min.z,
        )
    }

    pub fn volume(&self) -> f32 {
        let size = self.size();
        size.x * size.y * size.z
    }

    /// Check if two bounding boxes are approximately equal within tolerance
    pub fn approx_eq(&self, other: &BoundingBox, tolerance: f32) -> bool {
        (self.min.x - other.min.x).abs() < tolerance
            && (self.min.y - other.min.y).abs() < tolerance
            && (self.min.z - other.min.z).abs() < tolerance
            && (self.max.x - other.max.x).abs() < tolerance
            && (self.max.y - other.max.y).abs() < tolerance
            && (self.max.z - other.max.z).abs() < tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounding_box() {
        let mut bbox = BoundingBox::empty();
        bbox.expand_to_include(&Point3::new(1.0, 2.0, 3.0));
        bbox.expand_to_include(&Point3::new(-1.0, -2.0, -3.0));
        
        assert_eq!(bbox.min, Point3::new(-1.0, -2.0, -3.0));
        assert_eq!(bbox.max, Point3::new(1.0, 2.0, 3.0));
        assert_eq!(bbox.center(), Point3::new(0.0, 0.0, 0.0));
    }
}

