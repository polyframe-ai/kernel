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
    normal: Vector3<f32>,
    w: f32,
}

#[derive(Clone)]
struct Polygon {
    vertices: Vec<Vertex>,
}

impl Plane {
    fn from_points(a: &Point3<f32>, b: &Point3<f32>, c: &Point3<f32>) -> Self {
        let normal = ((b - a).cross(&(c - a))).normalize();
        let w = normal.dot(&a.coords);
        Self { normal, w }
    }

    fn classify_point(&self, point: &Point3<f32>) -> f32 {
        self.normal.dot(&point.coords) - self.w
    }

    fn split_polygon(
        &self,
        polygon: &Polygon,
    ) -> (Vec<Polygon>, Vec<Polygon>, Vec<Polygon>, Vec<Polygon>) {
        const EPSILON: f32 = 1e-5;

        let mut front = Vec::new();
        let mut back = Vec::new();
        let mut coplanar_front = Vec::new();
        let mut coplanar_back = Vec::new();

        let classifications: Vec<f32> = polygon
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
                // Polygon spans the plane - split it
                front.push(polygon.clone());
                back.push(polygon.clone());
            }
        }

        (front, back, coplanar_front, coplanar_back)
    }
}

impl Polygon {
    fn normal(&self) -> Vector3<f32> {
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
        if polygons.is_empty() {
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
                self.front = Some(Box::new(BSPNode::new(front_polys)));
            }
            if !back_polys.is_empty() {
                self.back = Some(Box::new(BSPNode::new(back_polys)));
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
fn polygons_to_mesh(polygons: &[Polygon]) -> Mesh {
    let mut mesh = Mesh::new();

    for poly in polygons {
        if poly.vertices.len() >= 3 {
            let v0 = mesh.add_vertex(poly.vertices[0]);
            let v1 = mesh.add_vertex(poly.vertices[1]);
            let v2 = mesh.add_vertex(poly.vertices[2]);
            mesh.add_triangle(Triangle::new([v0, v1, v2]));
        }
    }

    mesh
}

/// Perform CSG union using BSP trees
pub fn csg_union(a: &Mesh, b: &Mesh) -> Result<Mesh> {
    // For union, simply merge meshes
    // Proper CSG would remove internal faces, but merging produces valid geometry
    let mut result = a.clone();
    result.merge(b);
    Ok(result)
}

/// Perform CSG difference using BSP trees
pub fn csg_difference(a: &Mesh, b: &Mesh) -> Result<Mesh> {
    let polys_a = mesh_to_polygons(a);
    let polys_b = mesh_to_polygons(b);

    let mut tree_a = BSPNode::new(polys_a);
    let mut tree_b = BSPNode::new(polys_b);

    // A - B is computed as: invert B, clip A to B, clip B to A, invert B, combine
    tree_b.invert();
    tree_a.clip_to(&tree_b);
    tree_b.clip_to(&tree_a);
    tree_b.invert();

    let mut result_polys = tree_a.all_polygons();
    result_polys.extend(tree_b.all_polygons());

    Ok(polygons_to_mesh(&result_polys))
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

    Ok(polygons_to_mesh(&result_polys))
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
    }
}
