// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Math utilities

use nalgebra::{Point3, Vector3};

/// Calculate the normal of a triangle given three vertices
pub fn calculate_triangle_normal(
    p0: &Point3<f32>,
    p1: &Point3<f32>,
    p2: &Point3<f32>,
) -> Vector3<f32> {
    let v1 = p1 - p0;
    let v2 = p2 - p0;
    v1.cross(&v2).normalize()
}

/// Check if two floats are approximately equal
pub fn approx_eq(a: f32, b: f32, epsilon: f32) -> bool {
    (a - b).abs() < epsilon
}

/// Clamp a value between min and max
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Linear interpolation
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Convert degrees to radians
pub fn deg_to_rad(deg: f32) -> f32 {
    deg * std::f32::consts::PI / 180.0
}

/// Convert radians to degrees
pub fn rad_to_deg(rad: f32) -> f32 {
    rad * 180.0 / std::f32::consts::PI
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approx_eq() {
        assert!(approx_eq(1.0, 1.0001, 0.001));
        assert!(!approx_eq(1.0, 1.1, 0.001));
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5.0, 0.0, 10.0), 5.0);
        assert_eq!(clamp(-5.0, 0.0, 10.0), 0.0);
        assert_eq!(clamp(15.0, 0.0, 10.0), 10.0);
    }

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
    }

    #[test]
    fn test_angle_conversion() {
        let deg = 180.0;
        let rad = deg_to_rad(deg);
        assert!(approx_eq(rad, std::f32::consts::PI, 0.0001));
        assert!(approx_eq(rad_to_deg(rad), deg, 0.0001));
    }
}
