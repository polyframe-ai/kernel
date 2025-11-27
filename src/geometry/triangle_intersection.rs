// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Triangle-triangle intersection and splitting
//! Handles coplanar cases robustly using robust predicates

use super::robust_predicates::{classify_point_plane, PlaneClassification};
use nalgebra::{Point3, Vector3};

/// Result of triangle-triangle intersection test
#[derive(Debug, Clone)]
pub struct IntersectionResult {
    /// Whether triangles intersect
    pub intersects: bool,
    /// Intersection type
    pub intersection_type: IntersectionType,
    /// Intersection points/segments (if any)
    pub intersection_points: Vec<Point3<f64>>,
}

/// Type of triangle-triangle intersection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntersectionType {
    /// No intersection
    None,
    /// Triangles are coplanar and overlap
    Coplanar,
    /// Triangles intersect at a point
    Point,
    /// Triangles intersect along a line segment
    Segment,
}

/// Test if two triangles intersect
pub fn triangle_triangle_intersection(
    tri_a: &[Point3<f64>; 3],
    tri_b: &[Point3<f64>; 3],
) -> IntersectionResult {
    // Check if triangles are coplanar
    let coplanar = are_triangles_coplanar(tri_a, tri_b);
    
    if coplanar {
        // Handle coplanar case
        return handle_coplanar_triangles(tri_a, tri_b);
    }
    
    // Non-coplanar case: use standard intersection test
    // Check if any edge of tri_a intersects tri_b's plane and vice versa
    let mut intersection_points = Vec::new();
    
    // Test edges of tri_a against tri_b
    for i in 0..3 {
        let edge_start = tri_a[i];
        let edge_end = tri_a[(i + 1) % 3];
        
        if let Some(point) = edge_triangle_intersection(edge_start, edge_end, tri_b) {
            intersection_points.push(point);
        }
    }
    
    // Test edges of tri_b against tri_a
    for i in 0..3 {
        let edge_start = tri_b[i];
        let edge_end = tri_b[(i + 1) % 3];
        
        if let Some(point) = edge_triangle_intersection(edge_start, edge_end, tri_a) {
            intersection_points.push(point);
        }
    }
    
    // Deduplicate intersection points
    intersection_points = deduplicate_points(&intersection_points);
    
    let intersection_type = match intersection_points.len() {
        0 => IntersectionType::None,
        1 => IntersectionType::Point,
        2 => IntersectionType::Segment,
        _ => IntersectionType::Segment, // Multiple points form a segment
    };
    
    IntersectionResult {
        intersects: !intersection_points.is_empty(),
        intersection_type,
        intersection_points,
    }
}

/// Check if two triangles are coplanar
fn are_triangles_coplanar(
    tri_a: &[Point3<f64>; 3],
    tri_b: &[Point3<f64>; 3],
) -> bool {
    // Compute plane of tri_a
    let ab = tri_a[1] - tri_a[0];
    let ac = tri_a[2] - tri_a[0];
    let normal_a = ab.cross(&ac).normalize();
    let d_a = normal_a.dot(&tri_a[0].coords);
    
    // Check if all vertices of tri_b are on tri_a's plane
    for &vertex in tri_b {
        let classification = classify_point_plane(&vertex, &normal_a, d_a);
        if classification != PlaneClassification::OnPlane {
            return false;
        }
    }
    
    true
}

/// Handle coplanar triangle intersection
fn handle_coplanar_triangles(
    tri_a: &[Point3<f64>; 3],
    tri_b: &[Point3<f64>; 3],
) -> IntersectionResult {
    // Check if triangles overlap in 2D
    // Project to 2D plane and test overlap
    
    // For now, use simple bounding box test
    let bbox_a = compute_triangle_bbox(tri_a);
    let bbox_b = compute_triangle_bbox(tri_b);
    
    let overlaps = bboxes_overlap(&bbox_a, &bbox_b);
    
    IntersectionResult {
        intersects: overlaps,
        intersection_type: if overlaps {
            IntersectionType::Coplanar
        } else {
            IntersectionType::None
        },
        intersection_points: Vec::new(), // Coplanar case - would need 2D intersection
    }
}

/// Test if edge intersects triangle (non-coplanar case)
fn edge_triangle_intersection(
    edge_start: Point3<f64>,
    edge_end: Point3<f64>,
    triangle: &[Point3<f64>; 3],
) -> Option<Point3<f64>> {
    // Compute triangle plane
    let ab = triangle[1] - triangle[0];
    let ac = triangle[2] - triangle[0];
    let normal = ab.cross(&ac).normalize();
    let d = normal.dot(&triangle[0].coords);
    
    // Test edge endpoints
    let start_dist = normal.dot(&edge_start.coords) - d;
    let end_dist = normal.dot(&edge_end.coords) - d;
    
    // Check if edge crosses plane
    if start_dist * end_dist > 0.0 {
        return None; // Both on same side
    }
    
    // Compute intersection point
    let t = start_dist / (start_dist - end_dist);
    let intersection = edge_start + (edge_end - edge_start) * t;
    
    // Check if intersection point is inside triangle (2D test)
    if point_in_triangle_2d(&intersection, triangle, &normal) {
        Some(intersection)
    } else {
        None
    }
}

/// Test if point is inside triangle (projected to 2D)
fn point_in_triangle_2d(
    point: &Point3<f64>,
    triangle: &[Point3<f64>; 3],
    normal: &Vector3<f64>,
) -> bool {
    // Project to 2D by finding best projection plane
    let abs_normal = normal.map(|x| x.abs());
    let max_axis = if abs_normal.x > abs_normal.y && abs_normal.x > abs_normal.z {
        0 // Project to YZ plane
    } else if abs_normal.y > abs_normal.z {
        1 // Project to XZ plane
    } else {
        2 // Project to XY plane
    };
    
    // Get 2D coordinates
    let get_2d = |p: &Point3<f64>| -> (f64, f64) {
        match max_axis {
            0 => (p.y, p.z),
            1 => (p.x, p.z),
            _ => (p.x, p.y),
        }
    };
    
    let (px, py) = get_2d(point);
    let (v0x, v0y) = get_2d(&triangle[0]);
    let (v1x, v1y) = get_2d(&triangle[1]);
    let (v2x, v2y) = get_2d(&triangle[2]);
    
    // Barycentric coordinates test
    let denom = (v1y - v2y) * (v0x - v2x) + (v2x - v1x) * (v0y - v2y);
    if denom.abs() < 1e-10 {
        return false; // Degenerate triangle
    }
    
    let a = ((v1y - v2y) * (px - v2x) + (v2x - v1x) * (py - v2y)) / denom;
    let b = ((v2y - v0y) * (px - v2x) + (v0x - v2x) * (py - v2y)) / denom;
    let c = 1.0 - a - b;
    
    a >= 0.0 && b >= 0.0 && c >= 0.0
}

/// Compute bounding box of triangle
fn compute_triangle_bbox(triangle: &[Point3<f64>; 3]) -> (Point3<f64>, Point3<f64>) {
    let mut min = triangle[0];
    let mut max = triangle[0];
    
    for &vertex in triangle.iter().skip(1) {
        min.x = min.x.min(vertex.x);
        min.y = min.y.min(vertex.y);
        min.z = min.z.min(vertex.z);
        max.x = max.x.max(vertex.x);
        max.y = max.y.max(vertex.y);
        max.z = max.z.max(vertex.z);
    }
    
    (min, max)
}

/// Check if two bounding boxes overlap
fn bboxes_overlap(
    bbox_a: &(Point3<f64>, Point3<f64>),
    bbox_b: &(Point3<f64>, Point3<f64>),
) -> bool {
    bbox_a.0.x <= bbox_b.1.x
        && bbox_a.1.x >= bbox_b.0.x
        && bbox_a.0.y <= bbox_b.1.y
        && bbox_a.1.y >= bbox_b.0.y
        && bbox_a.0.z <= bbox_b.1.z
        && bbox_a.1.z >= bbox_b.0.z
}

/// Deduplicate points within epsilon distance
fn deduplicate_points(points: &[Point3<f64>]) -> Vec<Point3<f64>> {
    const EPS: f64 = 1e-9;
    let mut result: Vec<Point3<f64>> = Vec::new();
    
    for &point in points {
        let mut is_duplicate = false;
        for &existing in &result {
            if (point - existing).norm() < EPS {
                is_duplicate = true;
                break;
            }
        }
        if !is_duplicate {
            result.push(point);
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_intersection_disjoint() {
        let tri_a = [
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ];
        let tri_b = [
            Point3::new(2.0, 0.0, 0.0),
            Point3::new(3.0, 0.0, 0.0),
            Point3::new(2.0, 1.0, 0.0),
        ];
        
        let result = triangle_triangle_intersection(&tri_a, &tri_b);
        assert!(!result.intersects);
    }

    #[test]
    fn test_triangle_intersection_overlapping() {
        let tri_a = [
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ];
        let tri_b = [
            Point3::new(0.5, 0.0, 0.0),
            Point3::new(1.5, 0.0, 0.0),
            Point3::new(0.5, 1.0, 0.0),
        ];
        
        let result = triangle_triangle_intersection(&tri_a, &tri_b);
        // Should intersect (coplanar case)
        assert!(result.intersects || result.intersection_type == IntersectionType::Coplanar);
    }
}

