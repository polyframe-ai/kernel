// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Face fragment classification for CSG operations
//! Determines if face fragments are inside, outside, or on boundary of solids

use super::{Mesh, robust_predicates};
use nalgebra::{Point3, Vector3};

/// Classification of a face fragment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Classification {
    /// Fragment is inside the other solid
    Inside,
    /// Fragment is outside the other solid
    Outside,
    /// Fragment is on the boundary (within epsilon)
    OnBoundary,
}

/// Classify a face fragment relative to a mesh
pub fn classify_face_fragment(
    face_vertices: &[Point3<f64>; 3],
    other_mesh: &Mesh,
) -> Classification {
    // Check multiple points on the triangle for more robust classification
    // This prevents misclassification when centroid is inside but triangle is partially outside
    let centroid = compute_robust_centroid(face_vertices);
    
    // For union operations, we want to be VERY conservative and keep triangles unless
    // we're ABSOLUTELY CERTAIN they're completely inside the other mesh
    
    // Check all vertices and centroid
    let mut all_inside = true;
    let mut any_on_boundary = false;
    let mut any_outside = false;
    
    // Check centroid
    let centroid_inside = is_point_inside_solid(&centroid, other_mesh);
    let centroid_on_boundary = is_point_on_boundary(&centroid, other_mesh);
    if !centroid_inside && !centroid_on_boundary {
        any_outside = true;
    }
    if centroid_on_boundary {
        any_on_boundary = true;
        all_inside = false; // If on boundary, can't be fully inside
    }
    if !centroid_inside && !centroid_on_boundary {
        all_inside = false;
    }
    
    // Check all vertices
    for vertex in face_vertices {
        let v_inside = is_point_inside_solid(vertex, other_mesh);
        let v_on_boundary = is_point_on_boundary(vertex, other_mesh);
        
        if !v_inside && !v_on_boundary {
            any_outside = true;
            all_inside = false;
        }
        if v_on_boundary {
            any_on_boundary = true;
            all_inside = false; // If on boundary, can't be fully inside
        }
        if !v_inside && !v_on_boundary {
            all_inside = false;
        }
    }
    
    // Classification rules for union (very conservative):
    // - If ANY point is on boundary -> OnBoundary (keep it)
    // - If ANY point is outside -> Outside (keep it)
    // - Only if ALL points are clearly inside (not on boundary, not outside) -> Inside (discard it)
    // - If we're not sure, default to keeping the triangle (very conservative)
    if any_on_boundary {
        Classification::OnBoundary
    } else if any_outside {
        Classification::Outside
    } else if all_inside {
        // Only classify as Inside if ALL points are clearly inside (not on boundary, not outside)
        Classification::Inside
    } else {
        // Fallback: if we're not sure, keep it (very conservative)
        // This handles edge cases where classification is uncertain
        Classification::Outside
    }
}

/// Compute robust centroid of triangle
fn compute_robust_centroid(vertices: &[Point3<f64>; 3]) -> Point3<f64> {
    // Use Kahan summation for better accuracy
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_z = 0.0;
    let mut c_x = 0.0;
    let mut c_y = 0.0;
    let mut c_z = 0.0;
    
    for &vertex in vertices {
        let y_x = vertex.x - c_x;
        let t_x = sum_x + y_x;
        c_x = (t_x - sum_x) - y_x;
        sum_x = t_x;
        
        let y_y = vertex.y - c_y;
        let t_y = sum_y + y_y;
        c_y = (t_y - sum_y) - y_y;
        sum_y = t_y;
        
        let y_z = vertex.z - c_z;
        let t_z = sum_z + y_z;
        c_z = (t_z - sum_z) - y_z;
        sum_z = t_z;
    }
    
    Point3::new(sum_x / 3.0, sum_y / 3.0, sum_z / 3.0)
}

/// Test if point is inside solid using ray casting with robust predicates
fn is_point_inside_solid(point: &Point3<f64>, mesh: &Mesh) -> bool {
    // Cast ray in +X direction and count intersections
    let ray_dir = Vector3::new(1.0, 0.0, 0.0);
    let mut intersection_count = 0;
    
    for tri in &mesh.triangles {
        let v0 = &mesh.vertices[tri.indices[0]].position;
        let v1 = &mesh.vertices[tri.indices[1]].position;
        let v2 = &mesh.vertices[tri.indices[2]].position;
        
        if ray_intersects_triangle_robust(point, &ray_dir, v0, v1, v2) {
            intersection_count += 1;
        }
    }
    
    // Odd number of intersections = inside
    intersection_count % 2 == 1
}

/// Test if point is on boundary (within epsilon of any triangle)
/// Made more lenient to catch more boundary cases
fn is_point_on_boundary(point: &Point3<f64>, mesh: &Mesh) -> bool {
    const EPS: f64 = 1e-5; // Increased from 1e-6 to be more lenient
    
    for tri in &mesh.triangles {
        let v0 = &mesh.vertices[tri.indices[0]].position;
        let v1 = &mesh.vertices[tri.indices[1]].position;
        let v2 = &mesh.vertices[tri.indices[2]].position;
        
        // Compute distance to triangle plane
        let ab = v1 - v0;
        let ac = v2 - v0;
        let normal = ab.cross(&ac);
        let normal_len = normal.norm();
        
        if normal_len < 1e-10 {
            continue; // Degenerate triangle
        }
        
        let normal = normal / normal_len;
        let d = normal.dot(&v0.coords);
        
        let dist = (normal.dot(&point.coords) - d).abs();
        
        if dist < EPS {
            // Check if point projects onto triangle (with some tolerance)
            if point_in_triangle_robust(point, v0, v1, v2, &normal) {
                return true;
            }
        }
    }
    
    false
}

/// Robust ray-triangle intersection test
fn ray_intersects_triangle_robust(
    origin: &Point3<f64>,
    direction: &Vector3<f64>,
    v0: &Point3<f64>,
    v1: &Point3<f64>,
    v2: &Point3<f64>,
) -> bool {
    const EPS: f64 = 1e-9;
    
    let edge1 = v1 - v0;
    let edge2 = v2 - v0;
    let h = direction.cross(&edge2);
    let a = edge1.dot(&h);
    
    if a.abs() < EPS {
        return false; // Ray parallel to triangle
    }
    
    let f = 1.0 / a;
    let s = origin - v0;
    let u = f * s.dot(&h);
    
    if !(0.0..=1.0).contains(&u) {
        return false;
    }
    
    let q = s.cross(&edge1);
    let v = f * direction.dot(&q);
    
    if v < 0.0 || u + v > 1.0 {
        return false;
    }
    
    let t = f * edge2.dot(&q);
    t > EPS // Only count forward intersections
}

/// Robust point-in-triangle test
fn point_in_triangle_robust(
    point: &Point3<f64>,
    v0: &Point3<f64>,
    v1: &Point3<f64>,
    v2: &Point3<f64>,
    normal: &Vector3<f64>,
) -> bool {
    // Project to 2D plane
    let abs_normal = normal.map(|x| x.abs());
    let max_axis = if abs_normal.x > abs_normal.y && abs_normal.x > abs_normal.z {
        0
    } else if abs_normal.y > abs_normal.z {
        1
    } else {
        2
    };
    
    let get_2d = |p: &Point3<f64>| -> (f64, f64) {
        match max_axis {
            0 => (p.y, p.z),
            1 => (p.x, p.z),
            _ => (p.x, p.y),
        }
    };
    
    let (px, py) = get_2d(point);
    let (v0x, v0y) = get_2d(v0);
    let (v1x, v1y) = get_2d(v1);
    let (v2x, v2y) = get_2d(v2);
    
    // Barycentric coordinates
    let denom = (v1y - v2y) * (v0x - v2x) + (v2x - v1x) * (v0y - v2y);
    if denom.abs() < 1e-10 {
        return false;
    }
    
    let a = ((v1y - v2y) * (px - v2x) + (v2x - v1x) * (py - v2y)) / denom;
    let b = ((v2y - v0y) * (px - v2x) + (v0x - v2x) * (py - v2y)) / denom;
    let c = 1.0 - a - b;
    
    a >= -1e-9 && b >= -1e-9 && c >= -1e-9
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Primitive;

    #[test]
    fn test_classify_face_fragment() {
        let mesh = Primitive::cube(nalgebra::Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
        
        // Face outside the cube
        let face_outside = [
            Point3::new(20.0, 0.0, 0.0),
            Point3::new(21.0, 0.0, 0.0),
            Point3::new(20.0, 1.0, 0.0),
        ];
        
        let classification = classify_face_fragment(&face_outside, &mesh);
        assert_eq!(classification, Classification::Outside);
    }
}

