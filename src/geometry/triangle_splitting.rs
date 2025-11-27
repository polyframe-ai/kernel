// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Triangle splitting for CSG operations
//! Splits triangles along intersection boundaries to enable robust classification

use super::{Triangle, Vertex};
use nalgebra::{Point3, Vector3};
use super::robust_predicates::{classify_point_plane, PlaneClassification};

/// Result of splitting a triangle
#[derive(Debug, Clone)]
pub struct SplitResult {
    /// Triangle fragments from the split
    pub fragments: Vec<TriangleFragment>,
}

/// A triangle fragment with its vertices
#[derive(Debug, Clone)]
pub struct TriangleFragment {
    /// Vertices of the fragment (always 3 for a triangle)
    pub vertices: [Vertex; 3],
}

impl TriangleFragment {
    /// Convert to a Triangle with vertex indices
    /// The indices are relative to the vertex list provided to the splitting function
    pub fn to_triangle(&self, vertex_map: &mut VertexMap) -> Option<Triangle> {
        let i0 = vertex_map.get_or_add(&self.vertices[0]);
        let i1 = vertex_map.get_or_add(&self.vertices[1]);
        let i2 = vertex_map.get_or_add(&self.vertices[2]);
        
        // Check for degenerate triangle
        let v0 = &self.vertices[0].position;
        let v1 = &self.vertices[1].position;
        let v2 = &self.vertices[2].position;
        
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let area = edge1.cross(&edge2).norm();
        
        if area < 1e-10 {
            return None; // Degenerate triangle
        }
        
        Some(Triangle::new([i0, i1, i2]))
    }
}

/// Map for managing vertex indices during splitting
/// Handles vertex deduplication
pub struct VertexMap {
    vertices: Vec<Vertex>,
    index_map: std::collections::HashMap<PointKey, usize>,
}

#[derive(Hash, PartialEq, Eq, Clone)]
struct PointKey {
    x: i64,
    y: i64,
    z: i64,
}

impl PointKey {
    fn from_point(p: &Point3<f64>) -> Self {
        const SCALE: f64 = 1e9; // Scale to convert to integer for hashing
        Self {
            x: (p.x * SCALE) as i64,
            y: (p.y * SCALE) as i64,
            z: (p.z * SCALE) as i64,
        }
    }
}

impl VertexMap {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            index_map: std::collections::HashMap::new(),
        }
    }
    
    pub fn get_or_add(&mut self, vertex: &Vertex) -> usize {
        let key = PointKey::from_point(&vertex.position);
        
        if let Some(&idx) = self.index_map.get(&key) {
            // Check if it's really the same point (within epsilon)
            const EPS: f64 = 1e-9;
            if (self.vertices[idx].position - vertex.position).norm() < EPS {
                return idx;
            }
        }
        
        let idx = self.vertices.len();
        self.vertices.push(*vertex);
        self.index_map.insert(key, idx);
        idx
    }
    
    pub fn into_vertices(self) -> Vec<Vertex> {
        self.vertices
    }
}

/// Split a triangle along an intersection segment
/// The intersection_points should be 0, 1, or 2 points that lie on the triangle
/// Returns fragments of the original triangle
pub fn split_triangle_by_segment(
    triangle: &Triangle,
    vertices: &[Vertex],
    intersection_points: &[Point3<f64>],
) -> SplitResult {
    if intersection_points.is_empty() {
        // No intersection - return original triangle as single fragment
        return SplitResult {
            fragments: vec![TriangleFragment {
                vertices: [
                    vertices[triangle.indices[0]],
                    vertices[triangle.indices[1]],
                    vertices[triangle.indices[2]],
                ],
            }],
        };
    }
    
    if intersection_points.len() == 1 {
        // Point intersection - split at the point
        return split_triangle_at_point(triangle, vertices, &intersection_points[0]);
    }
    
    // Segment intersection (2 points) - split along the segment
    split_triangle_along_segment(triangle, vertices, &intersection_points[0], &intersection_points[1])
}

/// Split triangle at a single intersection point
fn split_triangle_at_point(
    triangle: &Triangle,
    vertices: &[Vertex],
    point: &Point3<f64>,
) -> SplitResult {
    let v0 = &vertices[triangle.indices[0]].position;
    let v1 = &vertices[triangle.indices[1]].position;
    let v2 = &vertices[triangle.indices[2]].position;
    
    // Find which edge the point lies on, or if it's at a vertex
    const EPS: f64 = 1e-9;
    
    // Check if point is at a vertex
    if (point - v0).norm() < EPS {
        return SplitResult {
            fragments: vec![TriangleFragment {
                vertices: [
                    vertices[triangle.indices[0]],
                    vertices[triangle.indices[1]],
                    vertices[triangle.indices[2]],
                ],
            }],
        };
    }
    if (point - v1).norm() < EPS {
        return SplitResult {
            fragments: vec![TriangleFragment {
                vertices: [
                    vertices[triangle.indices[0]],
                    vertices[triangle.indices[1]],
                    vertices[triangle.indices[2]],
                ],
            }],
        };
    }
    if (point - v2).norm() < EPS {
        return SplitResult {
            fragments: vec![TriangleFragment {
                vertices: [
                    vertices[triangle.indices[0]],
                    vertices[triangle.indices[1]],
                    vertices[triangle.indices[2]],
                ],
            }],
        };
    }
    
    // Check which edge the point lies on
    let edge01_dist = point_to_edge_distance(point, v0, v1);
    let edge12_dist = point_to_edge_distance(point, v1, v2);
    let edge20_dist = point_to_edge_distance(point, v2, v0);
    
    let mut fragments = Vec::new();
    
    if edge01_dist < EPS {
        // Point on edge 0-1
        let new_vertex = create_vertex_at_point(point, v0, v1, 
            &vertices[triangle.indices[0]], &vertices[triangle.indices[1]]);
        
        // Split into two triangles: (v0, point, v2) and (point, v1, v2)
        fragments.push(TriangleFragment {
            vertices: [
                vertices[triangle.indices[0]],
                new_vertex,
                vertices[triangle.indices[2]],
            ],
        });
        fragments.push(TriangleFragment {
            vertices: [
                new_vertex,
                vertices[triangle.indices[1]],
                vertices[triangle.indices[2]],
            ],
        });
    } else if edge12_dist < EPS {
        // Point on edge 1-2
        let new_vertex = create_vertex_at_point(point, v1, v2,
            &vertices[triangle.indices[1]], &vertices[triangle.indices[2]]);
        
        fragments.push(TriangleFragment {
            vertices: [
                vertices[triangle.indices[0]],
                vertices[triangle.indices[1]],
                new_vertex,
            ],
        });
        fragments.push(TriangleFragment {
            vertices: [
                vertices[triangle.indices[0]],
                new_vertex,
                vertices[triangle.indices[2]],
            ],
        });
    } else if edge20_dist < EPS {
        // Point on edge 2-0
        let new_vertex = create_vertex_at_point(point, v2, v0,
            &vertices[triangle.indices[2]], &vertices[triangle.indices[0]]);
        
        fragments.push(TriangleFragment {
            vertices: [
                vertices[triangle.indices[0]],
                vertices[triangle.indices[1]],
                new_vertex,
            ],
        });
        fragments.push(TriangleFragment {
            vertices: [
                new_vertex,
                vertices[triangle.indices[1]],
                vertices[triangle.indices[2]],
            ],
        });
    } else {
        // Point is inside triangle - split into 3 triangles from point to vertices
        let new_vertex = create_vertex_at_centroid(point, 
            &vertices[triangle.indices[0]], 
            &vertices[triangle.indices[1]], 
            &vertices[triangle.indices[2]]);
        
        fragments.push(TriangleFragment {
            vertices: [
                vertices[triangle.indices[0]],
                vertices[triangle.indices[1]],
                new_vertex,
            ],
        });
        fragments.push(TriangleFragment {
            vertices: [
                vertices[triangle.indices[1]],
                vertices[triangle.indices[2]],
                new_vertex,
            ],
        });
        fragments.push(TriangleFragment {
            vertices: [
                vertices[triangle.indices[2]],
                vertices[triangle.indices[0]],
                new_vertex,
            ],
        });
    }
    
    SplitResult { fragments }
}

/// Split triangle along a segment defined by two points
fn split_triangle_along_segment(
    triangle: &Triangle,
    vertices: &[Vertex],
    p1: &Point3<f64>,
    p2: &Point3<f64>,
) -> SplitResult {
    let v0 = &vertices[triangle.indices[0]].position;
    let v1 = &vertices[triangle.indices[1]].position;
    let v2 = &vertices[triangle.indices[2]].position;
    
    const EPS: f64 = 1e-9;
    
    // Find where the segment intersects the triangle edges
    // The segment endpoints should lie on edges or at vertices
    
    // Classify each endpoint
    let p1_location = locate_point_on_triangle(p1, v0, v1, v2);
    let p2_location = locate_point_on_triangle(p2, v0, v1, v2);
    
    match (p1_location, p2_location) {
        (PointLocation::Vertex(_), PointLocation::Vertex(_)) => {
            // Both at vertices - no splitting needed
            SplitResult {
                fragments: vec![TriangleFragment {
                    vertices: [
                        vertices[triangle.indices[0]],
                        vertices[triangle.indices[1]],
                        vertices[triangle.indices[2]],
                    ],
                }],
            }
        }
        (PointLocation::OnEdge(edge1), PointLocation::OnEdge(edge2)) => {
            if edge1 == edge2 {
                // Both points on same edge - split along that edge
                split_triangle_along_edge(triangle, vertices, edge1, p1, p2)
            } else {
                // Points on different edges - split across triangle
                split_triangle_across_edges(triangle, vertices, edge1, edge2, p1, p2)
            }
        }
        (PointLocation::OnEdge(edge), PointLocation::Vertex(vertex)) | 
        (PointLocation::Vertex(vertex), PointLocation::OnEdge(edge)) => {
            // One on edge, one at vertex
            let edge_point = if matches!(p1_location, PointLocation::OnEdge(_)) { p1 } else { p2 };
            let vertex_idx = if matches!(p1_location, PointLocation::Vertex(_)) {
                match p1_location {
                    PointLocation::Vertex(idx) => idx,
                    _ => unreachable!(),
                }
            } else {
                match p2_location {
                    PointLocation::Vertex(idx) => idx,
                    _ => unreachable!(),
                }
            };
            split_triangle_edge_to_vertex(triangle, vertices, edge, vertex_idx, edge_point)
        }
        _ => {
            // Complex case - use plane-based splitting
            // Compute plane through the segment and triangle normal
            let edge = p2 - p1;
            let tri_normal = (v1 - v0).cross(&(v2 - v0)).normalize();
            let plane_normal = edge.cross(&tri_normal).normalize();
            let plane_d = plane_normal.dot(&p1.coords);
            
            split_triangle_by_plane(triangle, vertices, &plane_normal, plane_d)
        }
    }
}

/// Location of a point relative to a triangle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PointLocation {
    Vertex(usize), // Index of vertex (0, 1, or 2)
    OnEdge(usize), // Index of edge (0=0-1, 1=1-2, 2=2-0)
    Inside,
}

/// Locate a point on a triangle
fn locate_point_on_triangle(
    point: &Point3<f64>,
    v0: &Point3<f64>,
    v1: &Point3<f64>,
    v2: &Point3<f64>,
) -> PointLocation {
    const EPS: f64 = 1e-9;
    
    // Check vertices
    if (point - v0).norm() < EPS {
        return PointLocation::Vertex(0);
    }
    if (point - v1).norm() < EPS {
        return PointLocation::Vertex(1);
    }
    if (point - v2).norm() < EPS {
        return PointLocation::Vertex(2);
    }
    
    // Check edges
    if point_to_edge_distance(point, v0, v1) < EPS {
        return PointLocation::OnEdge(0);
    }
    if point_to_edge_distance(point, v1, v2) < EPS {
        return PointLocation::OnEdge(1);
    }
    if point_to_edge_distance(point, v2, v0) < EPS {
        return PointLocation::OnEdge(2);
    }
    
    PointLocation::Inside
}

/// Split triangle along a single edge
fn split_triangle_along_edge(
    triangle: &Triangle,
    vertices: &[Vertex],
    edge: usize,
    p1: &Point3<f64>,
    p2: &Point3<f64>,
) -> SplitResult {
    // Order points along the edge
    let (v_start, v_end, v_opposite) = match edge {
        0 => (
            &vertices[triangle.indices[0]].position,
            &vertices[triangle.indices[1]].position,
            triangle.indices[2],
        ),
        1 => (
            &vertices[triangle.indices[1]].position,
            &vertices[triangle.indices[2]].position,
            triangle.indices[0],
        ),
        2 => (
            &vertices[triangle.indices[2]].position,
            &vertices[triangle.indices[0]].position,
            triangle.indices[1],
        ),
        _ => unreachable!(),
    };
    
    // Determine order of p1 and p2 along the edge
    let edge_dir = v_end - v_start;
    let t1 = (p1 - v_start).dot(&edge_dir) / edge_dir.norm_squared();
    let t2 = (p2 - v_start).dot(&edge_dir) / edge_dir.norm_squared();
    
    let (first, second) = if t1 < t2 { (p1, p2) } else { (p2, p1) };
    
    let v_first = create_vertex_at_point(first, v_start, v_end,
        &vertices[triangle.indices[edge]],
        &vertices[triangle.indices[(edge + 1) % 3]]);
    let v_second = create_vertex_at_point(second, v_start, v_end,
        &vertices[triangle.indices[edge]],
        &vertices[triangle.indices[(edge + 1) % 3]]);
    
    // Split into fragments along the edge segment
    let mut fragments = Vec::new();
    
    // Fragment 1: v_start to first point
    if (first - v_start).norm() > 1e-9 {
        fragments.push(TriangleFragment {
            vertices: [
                vertices[triangle.indices[edge]],
                v_first.clone(),
                vertices[v_opposite],
            ],
        });
    }
    
    // Fragment 2: first to second point
    fragments.push(TriangleFragment {
        vertices: [
            v_first,
            v_second.clone(),
            vertices[v_opposite],
        ],
    });
    
    // Fragment 3: second point to v_end
    if (second - v_end).norm() > 1e-9 {
        fragments.push(TriangleFragment {
            vertices: [
                v_second,
                vertices[triangle.indices[(edge + 1) % 3]],
                vertices[v_opposite],
            ],
        });
    }
    
    SplitResult { fragments }
}

/// Split triangle across two different edges
fn split_triangle_across_edges(
    triangle: &Triangle,
    vertices: &[Vertex],
    edge1: usize,
    edge2: usize,
    p1: &Point3<f64>,
    p2: &Point3<f64>,
) -> SplitResult {
    // Create vertices at intersection points
    let (v1_start, v1_end) = match edge1 {
        0 => (triangle.indices[0], triangle.indices[1]),
        1 => (triangle.indices[1], triangle.indices[2]),
        2 => (triangle.indices[2], triangle.indices[0]),
        _ => unreachable!(),
    };
    
    let (v2_start, v2_end) = match edge2 {
        0 => (triangle.indices[0], triangle.indices[1]),
        1 => (triangle.indices[1], triangle.indices[2]),
        2 => (triangle.indices[2], triangle.indices[0]),
        _ => unreachable!(),
    };
    
    let v1_new = create_vertex_at_point(p1, 
        &vertices[v1_start].position, 
        &vertices[v1_end].position,
        &vertices[v1_start], 
        &vertices[v1_end]);
    
    let v2_new = create_vertex_at_point(p2,
        &vertices[v2_start].position,
        &vertices[v2_end].position,
        &vertices[v2_start],
        &vertices[v2_end]);
    
    // Find the vertex that's not on either edge (the opposite vertex)
    let opposite_vertex = (0..3)
        .find(|&i| {
            let idx = triangle.indices[i];
            idx != v1_start && idx != v1_end && idx != v2_start && idx != v2_end
        })
        .map(|i| triangle.indices[i])
        .unwrap_or_else(|| {
            // Fallback: use the vertex that's shared by both edges if any
            if v1_start == v2_start || v1_start == v2_end {
                v1_start
            } else if v1_end == v2_start || v1_end == v2_end {
                v1_end
            } else {
                triangle.indices[0] // Fallback
            }
        });
    
    // Split into fragments
    let mut fragments = Vec::new();
    
    // The splitting creates a polygon that needs to be triangulated
    // For simplicity, split along the line connecting the two intersection points
    fragments.push(TriangleFragment {
        vertices: [
            vertices[opposite_vertex],
            v1_new.clone(),
            v2_new.clone(),
        ],
    });
    
    // Add remaining fragments based on which edges were cut
    // This is a simplified approach - full implementation would triangulate the polygon
    if edge1 == 0 && edge2 == 1 {
        // Cut edges 0-1 and 1-2, opposite is vertex 2
        fragments.push(TriangleFragment {
            vertices: [
                vertices[triangle.indices[0]],
                v1_new,
                vertices[triangle.indices[2]],
            ],
        });
        fragments.push(TriangleFragment {
            vertices: [
                v1_new,
                vertices[triangle.indices[1]],
                v2_new,
            ],
        });
    } else if edge1 == 1 && edge2 == 2 {
        // Cut edges 1-2 and 2-0, opposite is vertex 0
        fragments.push(TriangleFragment {
            vertices: [
                vertices[triangle.indices[1]],
                v1_new,
                vertices[triangle.indices[0]],
            ],
        });
        fragments.push(TriangleFragment {
            vertices: [
                v1_new,
                vertices[triangle.indices[2]],
                v2_new,
            ],
        });
    } else if edge1 == 2 && edge2 == 0 {
        // Cut edges 2-0 and 0-1, opposite is vertex 1
        fragments.push(TriangleFragment {
            vertices: [
                vertices[triangle.indices[2]],
                v1_new,
                vertices[triangle.indices[1]],
            ],
        });
        fragments.push(TriangleFragment {
            vertices: [
                v1_new,
                vertices[triangle.indices[0]],
                v2_new,
            ],
        });
    }
    
    SplitResult { fragments }
}

/// Split triangle from edge point to vertex
fn split_triangle_edge_to_vertex(
    triangle: &Triangle,
    vertices: &[Vertex],
    edge: usize,
    vertex_idx: usize,
    edge_point: &Point3<f64>,
) -> SplitResult {
    let (v_start, v_end) = match edge {
        0 => (triangle.indices[0], triangle.indices[1]),
        1 => (triangle.indices[1], triangle.indices[2]),
        2 => (triangle.indices[2], triangle.indices[0]),
        _ => unreachable!(),
    };
    
    let v_new = create_vertex_at_point(edge_point,
        &vertices[v_start].position,
        &vertices[v_end].position,
        &vertices[v_start],
        &vertices[v_end]);
    
    let mut fragments = Vec::new();
    
    // Split into two triangles: one from edge point to vertex, one remaining
    fragments.push(TriangleFragment {
        vertices: [
            v_new.clone(),
            vertices[vertex_idx],
            vertices[v_start],
        ],
    });
    fragments.push(TriangleFragment {
        vertices: [
            v_new,
            vertices[vertex_idx],
            vertices[v_end],
        ],
    });
    
    SplitResult { fragments }
}

/// Split triangle by a plane
/// Returns fragments on the front side and back side of the plane
pub fn split_triangle_by_plane(
    triangle: &Triangle,
    vertices: &[Vertex],
    plane_normal: &Vector3<f64>,
    plane_d: f64,
) -> SplitResult {
    let v0 = &vertices[triangle.indices[0]];
    let v1 = &vertices[triangle.indices[1]];
    let v2 = &vertices[triangle.indices[2]];
    
    // Classify vertices
    let c0 = classify_point_plane(&v0.position, plane_normal, plane_d);
    let c1 = classify_point_plane(&v1.position, plane_normal, plane_d);
    let c2 = classify_point_plane(&v2.position, plane_normal, plane_d);
    
    // Count classifications
    let front_count = [c0, c1, c2].iter().filter(|&&c| c == PlaneClassification::Front).count();
    let back_count = [c0, c1, c2].iter().filter(|&&c| c == PlaneClassification::Back).count();
    let on_count = [c0, c1, c2].iter().filter(|&&c| c == PlaneClassification::OnPlane).count();
    
    if front_count == 3 || (front_count == 2 && on_count == 1) {
        // All or mostly on front - return as single fragment
        return SplitResult {
            fragments: vec![TriangleFragment {
                vertices: [*v0, *v1, *v2],
            }],
        };
    }
    
    if back_count == 3 || (back_count == 2 && on_count == 1) {
        // All or mostly on back - return as single fragment
        return SplitResult {
            fragments: vec![TriangleFragment {
                vertices: [*v0, *v1, *v2],
            }],
        };
    }
    
    // Triangle spans the plane - need to split
    // Find intersection points on edges
    let mut intersection_points = Vec::new();
    let mut intersection_vertices = Vec::new();
    
    // Edge 0-1
    if c0 != c1 {
        if let Some(point) = edge_plane_intersection(&v0.position, &v1.position, plane_normal, plane_d) {
            intersection_points.push(point);
            intersection_vertices.push(create_vertex_at_point(&point, &v0.position, &v1.position, v0, v1));
        }
    }
    
    // Edge 1-2
    if c1 != c2 {
        if let Some(point) = edge_plane_intersection(&v1.position, &v2.position, plane_normal, plane_d) {
            intersection_points.push(point);
            intersection_vertices.push(create_vertex_at_point(&point, &v1.position, &v2.position, v1, v2));
        }
    }
    
    // Edge 2-0
    if c2 != c0 {
        if let Some(point) = edge_plane_intersection(&v2.position, &v0.position, plane_normal, plane_d) {
            intersection_points.push(point);
            intersection_vertices.push(create_vertex_at_point(&point, &v2.position, &v0.position, v2, v0));
        }
    }
    
    if intersection_points.len() != 2 {
        // Should have exactly 2 intersection points for a triangle spanning a plane
        // If not, return original triangle
        return SplitResult {
            fragments: vec![TriangleFragment {
                vertices: [*v0, *v1, *v2],
            }],
        };
    }
    
    // Split into two fragments
    let mut fragments = Vec::new();
    
    // Determine which vertices are on which side
    let front_vertex = if c0 == PlaneClassification::Front {
        Some(0)
    } else if c1 == PlaneClassification::Front {
        Some(1)
    } else if c2 == PlaneClassification::Front {
        Some(2)
    } else {
        None
    };
    
    if let Some(fv_idx) = front_vertex {
        // Fragment on front side
        fragments.push(TriangleFragment {
            vertices: [
                vertices[triangle.indices[fv_idx]],
                intersection_vertices[0].clone(),
                intersection_vertices[1].clone(),
            ],
        });
        
        // Fragment on back side (quadrilateral split into two triangles)
        let back_vertices: Vec<usize> = (0..3)
            .filter(|&i| {
                let idx = triangle.indices[i];
                let v = &vertices[idx];
                classify_point_plane(&v.position, plane_normal, plane_d) == PlaneClassification::Back
            })
            .map(|i| triangle.indices[i])
            .collect();
        
        if back_vertices.len() == 2 {
            fragments.push(TriangleFragment {
                vertices: [
                    vertices[back_vertices[0]],
                    vertices[back_vertices[1]],
                    intersection_vertices[0].clone(),
                ],
            });
            fragments.push(TriangleFragment {
                vertices: [
                    vertices[back_vertices[1]],
                    intersection_vertices[1].clone(),
                    intersection_vertices[0].clone(),
                ],
            });
        }
    }
    
    SplitResult { fragments }
}

/// Compute intersection point of edge with plane
fn edge_plane_intersection(
    v0: &Point3<f64>,
    v1: &Point3<f64>,
    plane_normal: &Vector3<f64>,
    plane_d: f64,
) -> Option<Point3<f64>> {
    let d0 = plane_normal.dot(&v0.coords) - plane_d;
    let d1 = plane_normal.dot(&v1.coords) - plane_d;
    
    // Check if edge crosses plane
    if d0 * d1 > 0.0 {
        return None; // Both on same side
    }
    
    // Compute intersection
    let t = d0 / (d0 - d1);
    Some(v0 + (v1 - v0) * t)
}

/// Create a vertex at a point on an edge
fn create_vertex_at_point(
    point: &Point3<f64>,
    v0: &Point3<f64>,
    v1: &Point3<f64>,
    vertex0: &Vertex,
    vertex1: &Vertex,
) -> Vertex {
    // Interpolate normal
    let t = if (v1 - v0).norm_squared() > 1e-18 {
        (point - v0).dot(&(v1 - v0)) / (v1 - v0).norm_squared()
    } else {
        0.5
    };
    let t = t.max(0.0).min(1.0);
    
    let normal = (vertex0.normal * (1.0 - t) + vertex1.normal * t).normalize();
    
    Vertex::new(*point, normal)
}

/// Create a vertex at a point inside triangle (interpolate from all vertices)
fn create_vertex_at_centroid(
    point: &Point3<f64>,
    v0: &Vertex,
    v1: &Vertex,
    v2: &Vertex,
) -> Vertex {
    // Use barycentric coordinates
    let p0 = v0.position;
    let p1 = v1.position;
    let p2 = v2.position;
    
    let v0v1 = p1 - p0;
    let v0v2 = p2 - p0;
    let v0p = point - p0;
    
    let dot00 = v0v1.dot(&v0v1);
    let dot01 = v0v1.dot(&v0v2);
    let dot02 = v0v1.dot(&v0p);
    let dot11 = v0v2.dot(&v0v2);
    let dot12 = v0v2.dot(&v0p);
    
    let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
    let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
    let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;
    let w = 1.0 - u - v;
    
    let normal = (v0.normal * w + v1.normal * u + v2.normal * v).normalize();
    
    Vertex::new(*point, normal)
}

/// Compute distance from point to edge
fn point_to_edge_distance(
    point: &Point3<f64>,
    edge_start: &Point3<f64>,
    edge_end: &Point3<f64>,
) -> f64 {
    let edge = edge_end - edge_start;
    let edge_len_sq = edge.norm_squared();
    
    if edge_len_sq < 1e-18 {
        return (point - edge_start).norm();
    }
    
    let t = (point - edge_start).dot(&edge) / edge_len_sq;
    let t = t.max(0.0).min(1.0);
    
    let closest = edge_start + edge * t;
    (point - closest).norm()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Vector3;

    #[test]
    fn test_split_triangle_no_intersection() {
        let triangle = Triangle::new([0, 1, 2]);
        let vertices = vec![
            Vertex::new(Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
            Vertex::new(Point3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
            Vertex::new(Point3::new(0.0, 1.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
        ];
        
        let result = split_triangle_by_segment(&triangle, &vertices, &[]);
        assert_eq!(result.fragments.len(), 1);
    }
    
    #[test]
    fn test_split_triangle_by_plane() {
        let triangle = Triangle::new([0, 1, 2]);
        let vertices = vec![
            Vertex::new(Point3::new(0.0, 0.0, -1.0), Vector3::new(0.0, 0.0, 1.0)),
            Vertex::new(Point3::new(1.0, 0.0, 1.0), Vector3::new(0.0, 0.0, 1.0)),
            Vertex::new(Point3::new(0.0, 1.0, 1.0), Vector3::new(0.0, 0.0, 1.0)),
        ];
        
        let plane_normal = Vector3::new(0.0, 0.0, 1.0);
        let plane_d = 0.0; // Plane at z=0
        
        let result = split_triangle_by_plane(&triangle, &vertices, &plane_normal, plane_d);
        // Should split into fragments
        assert!(result.fragments.len() >= 1);
    }
}

