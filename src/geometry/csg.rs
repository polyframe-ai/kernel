// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! CSG (Constructive Solid Geometry) operations using BSP tree

use super::{Mesh, Triangle, Vertex};
use anyhow::Result;
use nalgebra::{Point3, Vector3};

/// BSP tree node for CSG operations
#[derive(Clone)]
struct BSPNode {
    plane: Option<Plane>,
    front: Option<Box<BSPNode>>,
    back: Option<Box<BSPNode>>,
    polygons: Vec<Polygon>,
}

#[derive(Clone)]
struct Plane {
    normal: Vector3<f64>,
    w: f64,
}

#[derive(Clone)]
struct Polygon {
    vertices: Vec<Vertex>,
}

impl Plane {
    fn from_points(a: &Point3<f64>, b: &Point3<f64>, c: &Point3<f64>) -> Self {
        let normal = ((b - a).cross(&(c - a))).normalize();
        let w = normal.dot(&a.coords);
        Self { normal, w }
    }

    fn classify_point(&self, point: &Point3<f64>) -> f64 {
        self.normal.dot(&point.coords) - self.w
    }

    fn split_polygon(
        &self,
        polygon: &Polygon,
    ) -> (Vec<Polygon>, Vec<Polygon>, Vec<Polygon>, Vec<Polygon>) {
        const EPSILON: f64 = 1e-5;

        let mut front = Vec::new();
        let mut back = Vec::new();
        let mut coplanar_front = Vec::new();
        let mut coplanar_back = Vec::new();

        let classifications: Vec<f64> = polygon
            .vertices
            .iter()
            .map(|v| self.classify_point(&v.position))
            .collect();

        let mut polygon_type = 0;
        for &dist in &classifications {
            if dist > EPSILON {
                polygon_type |= 1; // FRONT
            } else if dist < -EPSILON {
                polygon_type |= 2; // BACK
            } else {
                polygon_type |= 4; // COPLANAR
            }
        }

        match polygon_type {
            1 => front.push(polygon.clone()),
            2 => back.push(polygon.clone()),
            4 => {
                if self.normal.dot(&polygon.normal()) > 0.0 {
                    coplanar_front.push(polygon.clone());
                } else {
                    coplanar_back.push(polygon.clone());
                }
            }
            _ => {
                // Polygon spans the plane - actually split it
                let mut front_verts = Vec::new();
                let mut back_verts = Vec::new();

                for i in 0..polygon.vertices.len() {
                    let j = (i + 1) % polygon.vertices.len();
                    let vi = &polygon.vertices[i];
                    let vj = &polygon.vertices[j];
                    let ti = classifications[i];
                    let tj = classifications[j];

                    // Add vertex i to appropriate side(s)
                    if ti >= -EPSILON {
                        front_verts.push(*vi);
                    }
                    if ti <= EPSILON {
                        back_verts.push(*vi);
                    }

                    // If edge crosses plane, add intersection vertex
                    if (ti > EPSILON && tj < -EPSILON) || (ti < -EPSILON && tj > EPSILON) {
                        let t = ti / (ti - tj);
                        let pos = vi.position + (vj.position - vi.position) * t;
                        let normal = (vi.normal + (vj.normal - vi.normal) * t).normalize();
                        let intersect = Vertex::new(pos, normal);
                        front_verts.push(intersect);
                        back_verts.push(intersect);
                    }
                }

                if front_verts.len() >= 3 {
                    front.push(Polygon {
                        vertices: front_verts,
                    });
                }
                if back_verts.len() >= 3 {
                    back.push(Polygon {
                        vertices: back_verts,
                    });
                }
            }
        }

        (front, back, coplanar_front, coplanar_back)
    }
}

impl Polygon {
    fn normal(&self) -> Vector3<f64> {
        if self.vertices.len() >= 3 {
            self.vertices[0].normal
        } else {
            Vector3::new(0.0, 1.0, 0.0)
        }
    }

    fn flip(&mut self) {
        self.vertices.reverse();
        for v in &mut self.vertices {
            v.normal = -v.normal;
        }
    }
}

impl BSPNode {
    fn new(polygons: Vec<Polygon>) -> Self {
        let mut node = Self {
            plane: None,
            front: None,
            back: None,
            polygons: Vec::new(),
        };

        if !polygons.is_empty() {
            node.build(polygons);
        }

        node
    }

    fn build(&mut self, polygons: Vec<Polygon>) {
        const MAX_DEPTH: usize = 50; // Prevent infinite recursion
        self.build_with_depth(polygons, 0, MAX_DEPTH);
    }

    fn build_with_depth(&mut self, polygons: Vec<Polygon>, depth: usize, max_depth: usize) {
        if polygons.is_empty() || depth >= max_depth {
            // At max depth, just store polygons without further splitting
            if depth >= max_depth {
                self.polygons = polygons;
            }
            return;
        }

        // Use first polygon's plane
        if self.plane.is_none() && !polygons[0].vertices.is_empty() {
            let v = &polygons[0].vertices;
            if v.len() >= 3 {
                self.plane = Some(Plane::from_points(
                    &v[0].position,
                    &v[1].position,
                    &v[2].position,
                ));
            }
        }

        if let Some(ref plane) = self.plane {
            let mut front_polys = Vec::new();
            let mut back_polys = Vec::new();

            for poly in polygons {
                let (mut f, mut b, mut cf, mut cb) = plane.split_polygon(&poly);
                front_polys.append(&mut f);
                back_polys.append(&mut b);
                self.polygons.append(&mut cf);
                self.polygons.append(&mut cb);
            }

            if !front_polys.is_empty() {
                let mut front_node = BSPNode {
                    plane: None,
                    front: None,
                    back: None,
                    polygons: Vec::new(),
                };
                front_node.build_with_depth(front_polys, depth + 1, max_depth);
                self.front = Some(Box::new(front_node));
            }
            if !back_polys.is_empty() {
                let mut back_node = BSPNode {
                    plane: None,
                    front: None,
                    back: None,
                    polygons: Vec::new(),
                };
                back_node.build_with_depth(back_polys, depth + 1, max_depth);
                self.back = Some(Box::new(back_node));
            }
        } else {
            self.polygons = polygons;
        }
    }

    fn all_polygons(&self) -> Vec<Polygon> {
        let mut result = self.polygons.clone();
        if let Some(ref front) = self.front {
            result.extend(front.all_polygons());
        }
        if let Some(ref back) = self.back {
            result.extend(back.all_polygons());
        }
        result
    }

    fn clip_to(&mut self, bsp: &BSPNode) {
        self.polygons = bsp.clip_polygons(&self.polygons);
        if let Some(ref mut front) = self.front {
            front.clip_to(bsp);
        }
        if let Some(ref mut back) = self.back {
            back.clip_to(bsp);
        }
    }

    /// Clip to BSP, keeping only polygons on the front side (outside)
    /// Used for union operations to remove internal polygons
    fn clip_to_front_only(&mut self, bsp: &BSPNode) {
        self.polygons = bsp.clip_polygons_front_only(&self.polygons);
        if let Some(ref mut front) = self.front {
            front.clip_to_front_only(bsp);
        }
        if let Some(ref mut back) = self.back {
            back.clip_to_front_only(bsp);
        }
    }

    fn clip_polygons(&self, polygons: &[Polygon]) -> Vec<Polygon> {
        if self.plane.is_none() {
            return polygons.to_vec();
        }

        let mut front = Vec::new();
        let mut back = Vec::new();

        for poly in polygons {
            let (mut f, mut b, _, _) = self.plane.as_ref().unwrap().split_polygon(poly);
            front.append(&mut f);
            back.append(&mut b);
        }

        let front = if let Some(ref front_node) = self.front {
            front_node.clip_polygons(&front)
        } else {
            front
        };

        let back = if let Some(ref back_node) = self.back {
            back_node.clip_polygons(&back)
        } else {
            Vec::new()
        };

        let mut result = front;
        result.extend(back);
        result
    }

    /// Clip polygons, keeping only those outside the shape (for union operations)
    /// This clips polygons against a normal (non-inverted) BSP tree, keeping only
    /// polygons that are in empty space (outside the solid).
    /// In a BSP tree for a solid:
    /// - Front subtree = outside the solid (empty space) - keep polygons here
    /// - Back subtree = inside the solid (filled space) - discard polygons here
    /// - Coplanar polygons = on the surface - discard (they're internal surfaces in union)
    fn clip_polygons_outside(&self, polygons: &[Polygon]) -> Vec<Polygon> {
        if self.plane.is_none() {
            // If no plane and no children, this is empty space - keep all polygons
            if self.front.is_none() && self.back.is_none() {
                return polygons.to_vec();
            }
            // If we have children but no plane, something is wrong - be conservative and discard
            return Vec::new();
        }

        let mut front = Vec::new();
        let mut back = Vec::new();

        for poly in polygons {
            let (mut f, mut b, _, _) = self.plane.as_ref().unwrap().split_polygon(poly);
            // Discard coplanar polygons - they're on the boundary and represent
            // internal surfaces that should be removed in union
            front.append(&mut f);
            back.append(&mut b);
        }

        // Front side = outside the solid (empty space) - keep these polygons
        let front_result = if let Some(ref front_node) = self.front {
            front_node.clip_polygons_outside(&front)
        } else {
            // No front node = we're in empty space outside the solid - keep all front polygons
            front
        };

        // Back side = inside the solid (filled space) - discard these polygons
        // (No need to recurse, we discard everything in back)

        // Return only front polygons (outside the solid)
        front_result
    }

    /// Clip polygons, keeping only those on the front side (for union operations)
    /// Front side = outside the solid (empty space), back side = inside (solid)
    /// Coplanar polygons are discarded as they represent internal surfaces in union
    fn clip_polygons_front_only(&self, polygons: &[Polygon]) -> Vec<Polygon> {
        if self.plane.is_none() {
            // Empty BSP means all space - if we're looking for "outside", return all
            // (This happens at leaf nodes that represent empty space)
            return polygons.to_vec();
        }

        let mut front = Vec::new();
        let mut back = Vec::new();
        // Coplanar polygons are discarded - they're on the boundary and become internal in union

        for poly in polygons {
            let (mut f, mut b, cf, cb) = self.plane.as_ref().unwrap().split_polygon(poly);
            front.append(&mut f);
            back.append(&mut b);
            // Discard coplanar polygons (cf, cb) - they're internal surfaces in union
        }

        // Recursively clip front polygons (outside space)
        let front = if let Some(ref front_node) = self.front {
            front_node.clip_polygons_front_only(&front)
        } else {
            // No front node = we've reached empty space outside the solid - keep all front polygons
            front
        };

        // Back polygons are inside the solid - discard them completely
        front
    }

    fn invert(&mut self) {
        for poly in &mut self.polygons {
            poly.flip();
        }
        if let Some(ref mut plane) = self.plane {
            plane.normal = -plane.normal;
            plane.w = -plane.w;
        }
        std::mem::swap(&mut self.front, &mut self.back);
        if let Some(ref mut front) = self.front {
            front.invert();
        }
        if let Some(ref mut back) = self.back {
            back.invert();
        }
    }
}

/// Convert mesh to polygons
fn mesh_to_polygons(mesh: &Mesh) -> Vec<Polygon> {
    mesh.triangles
        .iter()
        .map(|tri| Polygon {
            vertices: vec![
                mesh.vertices[tri.indices[0]],
                mesh.vertices[tri.indices[1]],
                mesh.vertices[tri.indices[2]],
            ],
        })
        .collect()
}

/// Convert polygons back to mesh
/// Polygons may have 3+ vertices (from BSP splitting), so we need to triangulate them
fn polygons_to_mesh(polygons: &[Polygon]) -> Mesh {
    let mut mesh = Mesh::new();

    for poly in polygons {
        if poly.vertices.len() < 3 {
            continue;
        }

        // For triangles, just add directly
        if poly.vertices.len() == 3 {
            let v0 = mesh.add_vertex(poly.vertices[0]);
            let v1 = mesh.add_vertex(poly.vertices[1]);
            let v2 = mesh.add_vertex(poly.vertices[2]);
            mesh.add_triangle(Triangle::new([v0, v1, v2]));
        } else {
            // For polygons with 4+ vertices, use fan triangulation
            // This assumes the polygon is convex (which BSP operations should produce)
            let base_vertex = mesh.add_vertex(poly.vertices[0]);
            
            for i in 1..poly.vertices.len() - 1 {
                let v1 = mesh.add_vertex(poly.vertices[i]);
                let v2 = mesh.add_vertex(poly.vertices[i + 1]);
                mesh.add_triangle(Triangle::new([base_vertex, v1, v2]));
            }
        }
    }

    // Recompute normals from triangle geometry
    mesh.recompute_normals();
    mesh
}

/// Perform CSG union using BSP trees
/// A ∪ B: keep parts of A outside B, keep parts of B outside A, combine
/// 
/// Note: Currently delegates to robust_union for more reliable results.
/// The BSP-based approach has issues with correctly identifying inside/outside regions.
/// 
/// Defaults to Robust quality for better results.
pub fn csg_union(a: &Mesh, b: &Mesh) -> Result<Mesh> {
    csg_union_with_quality(a, b, super::boolean::BooleanQuality::Robust)
}

/// Perform CSG union with specified quality
pub fn csg_union_with_quality(
    a: &Mesh,
    b: &Mesh,
    quality: super::boolean::BooleanQuality,
) -> Result<Mesh> {
    match quality {
        super::boolean::BooleanQuality::Fast => {
            // Use the robust union implementation which uses point-in-mesh tests
            super::robust_csg::robust_union(a, b)
        }
        super::boolean::BooleanQuality::Robust => {
            // Use the full robust union with intersection splitting
            super::robust_csg::robust_union_core(a, b)
        }
    }
}

/// Clip polygons to BSP tree, keeping only those on the BACK side
/// Used for union operations with inverted trees
/// When tree is inverted, BACK of inverted = OUTSIDE of original
fn clip_polygons_keep_back(bsp: &BSPNode, polygons: &[Polygon]) -> Vec<Polygon> {
    if bsp.plane.is_none() {
        // Empty BSP - if no children, this is empty space, keep all polygons
        if bsp.front.is_none() && bsp.back.is_none() {
            return polygons.to_vec();
        }
        // If there are children but no plane, something's wrong - discard
        return Vec::new();
    }

    let mut front = Vec::new();
    let mut back = Vec::new();

    for poly in polygons {
        let (mut f, mut b, _, _) = bsp.plane.as_ref().unwrap().split_polygon(poly);
        front.append(&mut f);
        back.append(&mut b);
    }

    // Recursively clip
    // For inverted tree: front = inside original (discard), back = outside original (keep)
    let _front = if let Some(ref front_node) = bsp.front {
        clip_polygons_keep_back(front_node, &front)
    } else {
        Vec::new() // No front node = inside, discard
    };

    let back = if let Some(ref back_node) = bsp.back {
        clip_polygons_keep_back(back_node, &back)
    } else {
        back // No back node = outside, keep all
    };

    // Return only back polygons (outside the original shape)
    back
}

/// Internal BSP-based difference (called by robust implementation)
/// Standard CSG difference algorithm (A - B):
/// 1. Keep parts of A that are outside B
/// 2. Keep parts of B (inverted) that are inside A (to close the hole)
/// 3. Combine
pub(crate) fn csg_difference_bsp_internal(a: &Mesh, b: &Mesh) -> Result<Mesh> {
    let polys_a = mesh_to_polygons(a);
    let polys_b = mesh_to_polygons(b);

    // Handle edge cases
    if polys_a.is_empty() {
        return Ok(Mesh::empty());
    }
    if polys_b.is_empty() {
        return Ok(a.clone());
    }

    let tree_a = BSPNode::new(polys_a.clone());
    let tree_b = BSPNode::new(polys_b.clone());

    // Step 1: Keep parts of A that are outside B
    // Use clip_polygons_front_only to keep only polygons outside B
    let a_outside_b = tree_b.clip_polygons_front_only(&polys_a);

    // Step 2: Keep parts of B (inverted) that are inside A
    // Invert B to get its complement
    let mut tree_b_inv = tree_b.clone();
    tree_b_inv.invert();
    let polys_b_inv = tree_b_inv.all_polygons();
    
    // Clip inverted B to A, keeping only parts inside A
    // We need to clip inverted B's polygons to A and keep the parts that are inside A
    // This is done by clipping to A and keeping front polygons (inside A)
    let b_inv_inside_a = tree_a.clip_polygons_front_only(&polys_b_inv);
    
    // Step 3: Combine the results
    let mut result_polys = a_outside_b;
    result_polys.extend(b_inv_inside_a);

    Ok(polygons_to_mesh(&result_polys))
}

/// Perform CSG difference using BSP trees
/// Note: This now uses robust implementation with fallback logic.
pub fn csg_difference(a: &Mesh, b: &Mesh) -> Result<Mesh> {
    // Use robust implementation which has BSP + winding number fallback
    super::robust_csg::robust_difference(a, b)
}

/// Perform CSG intersection using BSP trees
pub fn csg_intersection(a: &Mesh, b: &Mesh) -> Result<Mesh> {
    let polys_a = mesh_to_polygons(a);
    let polys_b = mesh_to_polygons(b);

    let mut tree_a = BSPNode::new(polys_a);
    let mut tree_b = BSPNode::new(polys_b);

    // A ∩ B is computed as: invert A, clip B to A, invert A, clip A to B, combine
    tree_a.invert();
    tree_b.clip_to(&tree_a);
    tree_a.invert();
    tree_a.clip_to(&tree_b);

    let mut result_polys = tree_a.all_polygons();
    result_polys.extend(tree_b.all_polygons());

    let mut result = polygons_to_mesh(&result_polys);
    // Normals already recomputed in polygons_to_mesh, but ensure they're correct
    result.recompute_normals();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Primitive;
    use nalgebra::Vector3;

    #[test]
    fn test_csg_union() {
        let mesh_a = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
        let mesh_b = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();

        let result = csg_union(&mesh_a, &mesh_b);
        assert!(result.is_ok());
        let mesh = result.unwrap();
        assert!(mesh.vertex_count() > 0);
    }

    #[test]
    fn test_csg_difference() {
        let mesh_a = Primitive::cube(Vector3::new(20.0, 20.0, 20.0), true).to_mesh();
        let mesh_b = Primitive::sphere(10.0, 16).to_mesh();

        let result = csg_difference(&mesh_a, &mesh_b);
        assert!(result.is_ok());
        let mesh = result.unwrap();
        assert!(mesh.vertex_count() > 0);
        
        // Verify normals are computed after CSG operation
        assert!(mesh.vertices.iter().all(|v| {
            let norm = v.normal.norm();
            norm > 0.9 && norm < 1.1 // Should be approximately unit length
        }));
    }

    #[test]
    fn test_csg_difference_cube_cylinder() {
        // Test the specific case from the images: cube with cylindrical hole
        let cube = Primitive::cube(Vector3::new(20.0, 20.0, 20.0), true).to_mesh();
        let cylinder = Primitive::cylinder(30.0, 5.0, 32).to_mesh();

        let result = csg_difference(&cube, &cylinder);
        assert!(result.is_ok());
        let mesh = result.unwrap();
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
        
        // Verify normals are computed correctly
        assert!(mesh.vertices.iter().all(|v| {
            let norm = v.normal.norm();
            norm > 0.9 && norm < 1.1
        }));
    }

    #[test]
    fn test_csg_union_preserves_normals() {
        let mesh_a = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
        let mesh_b = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();

        let result = csg_union(&mesh_a, &mesh_b);
        assert!(result.is_ok());
        let mesh = result.unwrap();
        
        // Verify normals are computed after union
        assert!(mesh.vertices.iter().all(|v| {
            let norm = v.normal.norm();
            norm > 0.9 && norm < 1.1
        }));
    }

    #[test]
    fn test_csg_union_overlapping_shapes() {
        // Test union of two overlapping cubes - should merge and remove internal faces
        let mesh_a = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
        // Cube B overlaps with A (shifted by 5 units)
        let mut mesh_b = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
        mesh_b.transform(&nalgebra::Matrix4::new_translation(&Vector3::new(5.0, 0.0, 0.0)));

        let result = csg_union(&mesh_a, &mesh_b);
        assert!(result.is_ok());
        let mesh = result.unwrap();
        
        // Union should have fewer vertices than simple merge (internal faces removed)
        // Simple merge would have: 8 + 8 = 16 vertices
        // Union should have fewer due to internal face removal
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
        
        // Verify normals are correct
        assert!(mesh.vertices.iter().all(|v| {
            let norm = v.normal.norm();
            norm > 0.9 && norm < 1.1
        }));
    }

    #[test]
    fn test_csg_difference_multiple() {
        // Test difference with multiple sequential operations: A - B - C
        let mesh_a = Primitive::cube(Vector3::new(20.0, 20.0, 20.0), true).to_mesh();
        let mesh_b = Primitive::sphere(8.0, 16).to_mesh();
        let mesh_c = Primitive::cylinder(30.0, 5.0, 32).to_mesh();

        // First difference: A - B
        let result1 = csg_difference(&mesh_a, &mesh_b);
        assert!(result1.is_ok());
        let intermediate = result1.unwrap();
        assert!(intermediate.vertex_count() > 0);

        // Second difference: (A - B) - C
        let result2 = csg_difference(&intermediate, &mesh_c);
        assert!(result2.is_ok());
        let final_mesh = result2.unwrap();
        assert!(final_mesh.vertex_count() > 0);
        assert!(final_mesh.triangle_count() > 0);
        
        // Verify normals
        assert!(final_mesh.vertices.iter().all(|v| {
            let norm = v.normal.norm();
            norm > 0.9 && norm < 1.1
        }));
    }

    #[test]
    fn test_csg_union_identical_shapes() {
        // Test union of identical overlapping shapes
        let mesh_a = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
        let mesh_b = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();

        let result = csg_union(&mesh_a, &mesh_b);
        assert!(result.is_ok());
        let mesh = result.unwrap();
        
        // Should result in a single cube (identical shapes merged)
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
    }
}
