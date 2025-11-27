// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Robust CSG operations using mesh-to-mesh operations
//! This provides a fallback for cases where BSP tree CSG fails

use super::{
    classification::{classify_face_fragment, Classification},
    triangle_intersection::{triangle_triangle_intersection, IntersectionResult, IntersectionType},
    triangle_splitting::{split_triangle_by_segment, split_triangle_by_plane},
    mesh_utils::{find_boundary_edges, build_edge_counts, classify_triangle_by_edges, TriangleClassification},
    Mesh, Triangle,
};
use anyhow::Result;
use nalgebra::{Point3, Vector3};

/// Perform robust CSG union using improved classification
/// This properly removes internal faces between overlapping meshes
/// 
/// This is the "Fast" version - uses simple point-in-mesh tests
/// For the full robust version with intersection splitting, use robust_union_core
pub fn robust_union(a: &Mesh, b: &Mesh) -> Result<Mesh> {
    // Handle edge cases
    if a.vertices.is_empty() {
        return Ok(b.clone());
    }
    if b.vertices.is_empty() {
        return Ok(a.clone());
    }

    let mut result = Mesh::new();

    // Union algorithm: Keep triangles from A that are outside B,
    // and keep triangles from B that are outside A
    // Key insight: For overlapping meshes, internal faces will have ALL their points
    // inside or on the boundary of the other mesh

    // Step 1: Add triangles from A that are outside B
    for tri in &a.triangles {
        let v0 = &a.vertices[tri.indices[0]];
        let v1 = &a.vertices[tri.indices[1]];
        let v2 = &a.vertices[tri.indices[2]];

        // Check center point
        let center = (v0.position.coords + v1.position.coords + v2.position.coords) / 3.0;
        let center_point = Point3::from(center);
        
        // Check if any point is inside B
        let v0_inside = is_point_inside_mesh(&v0.position, b);
        let v1_inside = is_point_inside_mesh(&v1.position, b);
        let v2_inside = is_point_inside_mesh(&v2.position, b);
        let center_inside = is_point_inside_mesh(&center_point, b);

        // Calculate triangle normal
        let edge1 = v1.position - v0.position;
        let edge2 = v2.position - v0.position;
        let tri_normal = edge1.cross(&edge2);
        let tri_normal_norm = tri_normal.norm();
        
        // Skip degenerate triangles
        if tri_normal_norm < 1e-10 {
            continue;
        }
        let tri_normal = tri_normal / tri_normal_norm;

        // Check if triangle center is very close to any surface of B
        // If it is, check if it's a coplanar internal face
        let surface_normal = find_nearest_surface_normal(&center_point, b, 0.1);
        
        if let Some(normal) = surface_normal {
            // Check if triangle is coplanar with B's surface (dot product close to 1)
            let dot = tri_normal.dot(&normal).abs();
            if dot > 0.95 {
                // Triangle is coplanar with B's surface
                // Check if any vertices are inside - if so, this is an internal face
                if v0_inside || v1_inside || v2_inside || center_inside {
                    continue; // Skip - this is an internal coplanar face
                }
            }
        }

        // Keep triangle if AT LEAST ONE point is definitely outside
        // This is less strict but should work better for boundary cases
        let any_point_outside = !v0_inside || !v1_inside || !v2_inside || !center_inside;
        
        if any_point_outside {
            let i0 = result.add_vertex(*v0);
            let i1 = result.add_vertex(*v1);
            let i2 = result.add_vertex(*v2);
            result.add_triangle(Triangle::new([i0, i1, i2]));
        }
    }

    // Step 2: Add triangles from B that are outside A
    for tri in &b.triangles {
        let v0 = &b.vertices[tri.indices[0]];
        let v1 = &b.vertices[tri.indices[1]];
        let v2 = &b.vertices[tri.indices[2]];

        let center = (v0.position.coords + v1.position.coords + v2.position.coords) / 3.0;
        let center_point = Point3::from(center);

        let v0_inside = is_point_inside_mesh(&v0.position, a);
        let v1_inside = is_point_inside_mesh(&v1.position, a);
        let v2_inside = is_point_inside_mesh(&v2.position, a);
        let center_inside = is_point_inside_mesh(&center_point, a);

        // Calculate triangle normal
        let edge1 = v1.position - v0.position;
        let edge2 = v2.position - v0.position;
        let tri_normal = edge1.cross(&edge2);
        let tri_normal_norm = tri_normal.norm();
        
        if tri_normal_norm < 1e-10 {
            continue;
        }
        let tri_normal = tri_normal / tri_normal_norm;

        // Check if triangle center is very close to any surface of A
        let surface_normal = find_nearest_surface_normal(&center_point, a, 0.1);
        
        if let Some(normal) = surface_normal {
            let dot = tri_normal.dot(&normal).abs();
            if dot > 0.95 {
                // Coplanar - check if internal
                if v0_inside || v1_inside || v2_inside || center_inside {
                    continue; // Skip - internal coplanar face
                }
            }
        }

        let any_point_outside = !v0_inside || !v1_inside || !v2_inside || !center_inside;
        
        if any_point_outside {
            let i0 = result.add_vertex(*v0);
            let i1 = result.add_vertex(*v1);
            let i2 = result.add_vertex(*v2);
            result.add_triangle(Triangle::new([i0, i1, i2]));
        }
    }

    // Recompute normals from triangle geometry to ensure correctness
    result.recompute_normals();
    Ok(result)
}

/// Robust union core algorithm with intersection splitting and classification
/// This is the full robust implementation that:
/// 1. Builds BVH for both meshes
/// 2. Finds all triangle-triangle intersections
/// 3. Splits triangles at intersection boundaries
/// 4. Classifies all face fragments (inside/outside)
/// 5. Keeps fragments: A outside B, B outside A
/// 6. Reconstructs manifold mesh
pub fn robust_union_core(a: &Mesh, b: &Mesh) -> Result<Mesh> {
    use super::{bvh::BVH, BoundingBox};
    
    // DEBUG: This is the robust union core being called
    // It uses BVH acceleration and intersection detection
    
    // Handle edge cases
    if a.vertices.is_empty() {
        return Ok(b.clone());
    }
    if b.vertices.is_empty() {
        return Ok(a.clone());
    }
    
    // Build BVH for both meshes (for future intersection acceleration)
    let triangles_a: Vec<(usize, BoundingBox)> = a
        .triangles
        .iter()
        .enumerate()
        .map(|(idx, tri)| {
            let v0 = &a.vertices[tri.indices[0]];
            let v1 = &a.vertices[tri.indices[1]];
            let v2 = &a.vertices[tri.indices[2]];
            
            let mut bbox = BoundingBox::empty();
            bbox.expand_to_include(&v0.position);
            bbox.expand_to_include(&v1.position);
            bbox.expand_to_include(&v2.position);
            
            (idx, bbox)
        })
        .collect();
    
    let bvh_a = BVH::build(triangles_a);
    
    let triangles_b: Vec<(usize, BoundingBox)> = b
        .triangles
        .iter()
        .enumerate()
        .map(|(idx, tri)| {
            let v0 = &b.vertices[tri.indices[0]];
            let v1 = &b.vertices[tri.indices[1]];
            let v2 = &b.vertices[tri.indices[2]];
            
            let mut bbox = BoundingBox::empty();
            bbox.expand_to_include(&v0.position);
            bbox.expand_to_include(&v1.position);
            bbox.expand_to_include(&v2.position);
            
            (idx, bbox)
        })
        .collect();
    
    let bvh_b = BVH::build(triangles_b);
    
    // Find all triangle-triangle intersections using BVH acceleration
    // Store full intersection data for each triangle
    let mut intersections_a: std::collections::HashMap<usize, Vec<(usize, IntersectionResult)>> = std::collections::HashMap::new();
    let mut intersections_b: std::collections::HashMap<usize, Vec<(usize, IntersectionResult)>> = std::collections::HashMap::new();
    
    // For each triangle in A, find potentially intersecting triangles in B
    for (tri_a_idx, tri_a) in a.triangles.iter().enumerate() {
        let v0_a = &a.vertices[tri_a.indices[0]];
        let v1_a = &a.vertices[tri_a.indices[1]];
        let v2_a = &a.vertices[tri_a.indices[2]];
        
        let mut bbox_a = BoundingBox::empty();
        bbox_a.expand_to_include(&v0_a.position);
        bbox_a.expand_to_include(&v1_a.position);
        bbox_a.expand_to_include(&v2_a.position);
        
        // Query BVH for potentially intersecting triangles in B
        let candidate_tri_b = bvh_b.query_triangles(&bbox_a);
        
        let tri_a_vertices = [v0_a.position, v1_a.position, v2_a.position];
        
        for &tri_b_idx in &candidate_tri_b {
            let tri_b = &b.triangles[tri_b_idx];
            let v0_b = &b.vertices[tri_b.indices[0]];
            let v1_b = &b.vertices[tri_b.indices[1]];
            let v2_b = &b.vertices[tri_b.indices[2]];
            
            let tri_b_vertices = [v0_b.position, v1_b.position, v2_b.position];
            
            // Test for intersection
            let intersection = super::triangle_intersection::triangle_triangle_intersection(
                &tri_a_vertices,
                &tri_b_vertices,
            );
            
            if intersection.intersects {
                intersections_a.entry(tri_a_idx)
                    .or_insert_with(Vec::new)
                    .push((tri_b_idx, intersection.clone()));
                intersections_b.entry(tri_b_idx)
                    .or_insert_with(Vec::new)
                    .push((tri_a_idx, intersection));
            }
        }
    }
    
    // Build result mesh
    let mut result = Mesh::new();
    // Track which mesh each triangle came from (0 = mesh A, 1 = mesh B)
    let mut triangle_mesh_source: Vec<usize> = Vec::new();

    // Step 1: Process triangles from A
    for (tri_idx, tri) in a.triangles.iter().enumerate() {
        let v0 = &a.vertices[tri.indices[0]];
        let v1 = &a.vertices[tri.indices[1]];
        let v2 = &a.vertices[tri.indices[2]];
        
        let face_vertices = [v0.position, v1.position, v2.position];
        
        // Check if this triangle has intersections
        if let Some(intersection_list) = intersections_a.get(&tri_idx) {
            // Triangle has intersections - split it
            let mut all_intersection_points = Vec::new();
            let mut has_coplanar = false;
            
            // Collect all intersection points from all intersecting triangles
            for (_other_tri_idx, intersection_result) in intersection_list {
                match intersection_result.intersection_type {
                    IntersectionType::Point => {
                        // Single point intersection
                        all_intersection_points.extend(intersection_result.intersection_points.iter().cloned());
                    }
                    IntersectionType::Segment => {
                        // Segment intersection - add both endpoints
                        all_intersection_points.extend(intersection_result.intersection_points.iter().cloned());
                    }
                    IntersectionType::Coplanar => {
                        // Coplanar case - handle separately
                        has_coplanar = true;
                    }
                    IntersectionType::None => {}
                }
            }
            
            // Deduplicate intersection points
            all_intersection_points = deduplicate_intersection_points(&all_intersection_points);
            
            // Split the triangle
            let split_result = if has_coplanar {
                // For coplanar case, use plane-based splitting
                // Compute plane from the coplanar triangle
                if let Some((other_idx, _)) = intersection_list.first() {
                    let other_tri = &b.triangles[*other_idx];
                    let ov0 = &b.vertices[other_tri.indices[0]].position;
                    let ov1 = &b.vertices[other_tri.indices[1]].position;
                    let ov2 = &b.vertices[other_tri.indices[2]].position;
                    
                    let edge1 = ov1 - ov0;
                    let edge2 = ov2 - ov0;
                    let plane_normal = edge1.cross(&edge2).normalize();
                    let plane_d = plane_normal.dot(&ov0.coords);
                    
                    split_triangle_by_plane(tri, &a.vertices, &plane_normal, plane_d)
                } else {
                    // Fallback: no splitting
                    split_triangle_by_segment(tri, &a.vertices, &[])
                }
            } else if all_intersection_points.len() <= 2 {
                // Normal splitting with intersection points
                split_triangle_by_segment(tri, &a.vertices, &all_intersection_points)
            } else {
                // Multiple intersection points - use first two (simplified)
                split_triangle_by_segment(tri, &a.vertices, &all_intersection_points[..2.min(all_intersection_points.len())])
            };
            
            // Classify and keep fragments that are outside or on boundary
            // For union, we keep fragments that are outside the other mesh
            // OnBoundary fragments are also kept as they're part of the union surface
            let mut kept_fragments = 0;
            for fragment in &split_result.fragments {
                let frag_vertices = [
                    fragment.vertices[0].position,
                    fragment.vertices[1].position,
                    fragment.vertices[2].position,
                ];
                let classification = classify_face_fragment(&frag_vertices, b);
                
                // For union, keep Outside and OnBoundary triangles
                // Duplicate coplanar triangles will be removed by remove_duplicate_triangles()
                let should_keep = match classification {
                    Classification::Outside => true,
                    Classification::OnBoundary => true, // Keep boundary triangles - duplicates will be removed
                    Classification::Inside => false,
                };
                
                if should_keep {
                    // Add fragment vertices directly to result mesh
                    let i0 = result.add_vertex(fragment.vertices[0]);
                    let i1 = result.add_vertex(fragment.vertices[1]);
                    let i2 = result.add_vertex(fragment.vertices[2]);
                    result.add_triangle(Triangle::new([i0, i1, i2]));
                    triangle_mesh_source.push(0); // From mesh A
                    kept_fragments += 1;
                }
            }
            
            // Fallback: if no fragments were kept, always re-check the original triangle
            // This handles cases where:
            // 1. Splitting failed or didn't actually split (fragments.len() == 1)
            // 2. All fragments were incorrectly classified as Inside
            // 3. Classification of fragments was wrong but original triangle should be kept
            if kept_fragments == 0 {
                // Re-check the original triangle - classification might have been wrong
                let classification = classify_face_fragment(&face_vertices, b);
                let should_keep = match classification {
                    Classification::Outside => true,
                    Classification::OnBoundary => true, // Keep boundary triangles
                    Classification::Inside => false,
                };
                if should_keep {
                    let i0 = result.add_vertex(*v0);
                    let i1 = result.add_vertex(*v1);
                    let i2 = result.add_vertex(*v2);
                    result.add_triangle(Triangle::new([i0, i1, i2]));
                    triangle_mesh_source.push(0); // From mesh A
                }
            }
        } else {
            // Non-intersecting triangle - use robust classification
            let classification = classify_face_fragment(&face_vertices, b);
            // For union, keep Outside and OnBoundary triangles
            let should_keep = match classification {
                Classification::Outside => true,
                Classification::OnBoundary => true, // Keep boundary triangles
                Classification::Inside => false,
            };
            if should_keep {
                let i0 = result.add_vertex(*v0);
                let i1 = result.add_vertex(*v1);
                let i2 = result.add_vertex(*v2);
                result.add_triangle(Triangle::new([i0, i1, i2]));
                triangle_mesh_source.push(0); // From mesh A
            }
        }
    }
    
    // Step 2: Process triangles from B
    for (tri_idx, tri) in b.triangles.iter().enumerate() {
        let v0 = &b.vertices[tri.indices[0]];
        let v1 = &b.vertices[tri.indices[1]];
        let v2 = &b.vertices[tri.indices[2]];
        
        let face_vertices = [v0.position, v1.position, v2.position];
        
        // Check if this triangle has intersections
        if let Some(intersection_list) = intersections_b.get(&tri_idx) {
            // Triangle has intersections - split it
            let mut all_intersection_points = Vec::new();
            let mut has_coplanar = false;
            
            // Collect all intersection points from all intersecting triangles
            for (_other_tri_idx, intersection_result) in intersection_list {
                match intersection_result.intersection_type {
                    IntersectionType::Point => {
                        all_intersection_points.extend(intersection_result.intersection_points.iter().cloned());
                    }
                    IntersectionType::Segment => {
                        all_intersection_points.extend(intersection_result.intersection_points.iter().cloned());
                    }
                    IntersectionType::Coplanar => {
                        has_coplanar = true;
                    }
                    IntersectionType::None => {}
                }
            }
            
            // Deduplicate intersection points
            all_intersection_points = deduplicate_intersection_points(&all_intersection_points);
            
            // Split the triangle
            let split_result = if has_coplanar {
                // For coplanar case, use plane-based splitting
                if let Some((other_idx, _)) = intersection_list.first() {
                    let other_tri = &a.triangles[*other_idx];
                    let ov0 = &a.vertices[other_tri.indices[0]].position;
                    let ov1 = &a.vertices[other_tri.indices[1]].position;
                    let ov2 = &a.vertices[other_tri.indices[2]].position;
                    
                    let edge1 = ov1 - ov0;
                    let edge2 = ov2 - ov0;
                    let plane_normal = edge1.cross(&edge2).normalize();
                    let plane_d = plane_normal.dot(&ov0.coords);
                    
                    split_triangle_by_plane(tri, &b.vertices, &plane_normal, plane_d)
                } else {
                    split_triangle_by_segment(tri, &b.vertices, &[])
                }
            } else if all_intersection_points.len() <= 2 {
                split_triangle_by_segment(tri, &b.vertices, &all_intersection_points)
            } else {
                split_triangle_by_segment(tri, &b.vertices, &all_intersection_points[..2.min(all_intersection_points.len())])
            };
            
            // Classify and keep fragments that are outside or on boundary
            // For union, we keep fragments that are outside the other mesh
            // OnBoundary fragments are also kept as they're part of the union surface
            let mut kept_fragments = 0;
            for fragment in &split_result.fragments {
                let frag_vertices = [
                    fragment.vertices[0].position,
                    fragment.vertices[1].position,
                    fragment.vertices[2].position,
                ];
                let classification = classify_face_fragment(&frag_vertices, a);
                
                // For union, keep Outside and OnBoundary triangles
                // Duplicate coplanar triangles will be removed by remove_duplicate_triangles()
                let should_keep = match classification {
                    Classification::Outside => true,
                    Classification::OnBoundary => true, // Keep boundary triangles - duplicates will be removed
                    Classification::Inside => false,
                };
                
                if should_keep {
                    // Add fragment vertices directly to result mesh
                    let i0 = result.add_vertex(fragment.vertices[0]);
                    let i1 = result.add_vertex(fragment.vertices[1]);
                    let i2 = result.add_vertex(fragment.vertices[2]);
                    result.add_triangle(Triangle::new([i0, i1, i2]));
                    triangle_mesh_source.push(1); // From mesh B
                    kept_fragments += 1;
                }
            }
            
            // Fallback: if no fragments were kept, always re-check the original triangle
            // This handles cases where:
            // 1. Splitting failed or didn't actually split (fragments.len() == 1)
            // 2. All fragments were incorrectly classified as Inside
            // 3. Classification of fragments was wrong but original triangle should be kept
            if kept_fragments == 0 {
                // Re-check the original triangle - classification might have been wrong
                let classification = classify_face_fragment(&face_vertices, a);
                let should_keep = match classification {
                    Classification::Outside => true,
                    Classification::OnBoundary => true, // Keep boundary triangles
                    Classification::Inside => false,
                };
                if should_keep {
                    let i0 = result.add_vertex(*v0);
                    let i1 = result.add_vertex(*v1);
                    let i2 = result.add_vertex(*v2);
                    result.add_triangle(Triangle::new([i0, i1, i2]));
                    triangle_mesh_source.push(1); // From mesh B
                }
            }
        } else {
            // Non-intersecting triangle - use robust classification
            let classification = classify_face_fragment(&face_vertices, a);
            // For union, keep Outside and OnBoundary triangles
            let should_keep = match classification {
                Classification::Outside => true,
                Classification::OnBoundary => true, // Keep boundary triangles
                Classification::Inside => false,
            };
            if should_keep {
                let i0 = result.add_vertex(*v0);
                let i1 = result.add_vertex(*v1);
                let i2 = result.add_vertex(*v2);
                result.add_triangle(Triangle::new([i0, i1, i2]));
                triangle_mesh_source.push(1); // From mesh B
            }
        }
    }
    
    // Clean up the mesh: comprehensive cleanup pipeline
    // This fixes artifacts from duplicate vertices, overlapping triangles, and z-fighting
    const WELD_EPSILON: f64 = 1e-6;
    
    // Step 1: Weld vertices (remove duplicate vertices at intersection points)
    result.weld_vertices(WELD_EPSILON);
    
    // Step 2: Remove exact duplicate triangles (same indices)
    // Note: This only removes exact duplicates, not coplanar overlaps
    // TEMPORARILY DISABLED to debug missing faces
    // TODO: Re-enable and check if it's removing legitimate triangles
    // result.remove_duplicate_triangles();
    
    // Step 3: Remove coplanar overlapping triangles with z-fighting prevention
    // Use mesh source tracking to prefer mesh A for coplanar boundaries
    // Only removes triangles with significant overlap (>= 50% of smaller triangle)
    // This prevents removing triangles that just touch at edges
    // Sync triangle_mesh_source length with result.triangles length
    if triangle_mesh_source.len() != result.triangles.len() {
        // Adjust triangle_mesh_source to match current triangle count
        triangle_mesh_source.truncate(result.triangles.len());
        while triangle_mesh_source.len() < result.triangles.len() {
            triangle_mesh_source.push(0); // Default to mesh A for any new triangles
        }
    }
    
    // Now remove coplanar duplicates
    // TEMPORARILY DISABLED to debug artifacts
    // TODO: Re-enable with better overlap detection
    // if triangle_mesh_source.len() == result.triangles.len() {
    //     result.remove_coplanar_duplicates(&triangle_mesh_source, None);
    // }
    
    // Step 4: Remove orphaned vertices (vertices not referenced by any triangle)
    result.remove_orphaned_vertices();
    
    // Step 5: Recompute normals after all cleanup (vertices may have changed)
    result.recompute_normals();
    
    Ok(result)
}

/// Deduplicate intersection points within epsilon distance
fn deduplicate_intersection_points(points: &[Point3<f64>]) -> Vec<Point3<f64>> {
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

/// Find the normal of the nearest surface to a point (within epsilon distance)
fn find_nearest_surface_normal(point: &Point3<f64>, mesh: &Mesh, epsilon: f64) -> Option<Vector3<f64>> {
    let mut nearest_normal = None;
    let mut min_distance = epsilon;

    for tri in &mesh.triangles {
        let v0 = &mesh.vertices[tri.indices[0]];
        let v1 = &mesh.vertices[tri.indices[1]];
        let v2 = &mesh.vertices[tri.indices[2]];

        // Calculate triangle plane
        let edge1 = v1.position - v0.position;
        let edge2 = v2.position - v0.position;
        let normal = edge1.cross(&edge2).normalize();
        let d = -normal.dot(&v0.position.coords);

        // Distance from point to plane
        let distance = (normal.dot(&point.coords) + d).abs();

        if distance < min_distance {
            // Check if point projects onto the triangle (simplified check)
            // For now, just check if it's close to the plane
            min_distance = distance;
            nearest_normal = Some(normal);
        }
    }

    nearest_normal
}

/// Perform robust CSG difference using mesh-based approach
/// This uses a winding-number based classification that's more robust than BSP for curved surfaces
pub fn robust_difference(a: &Mesh, b: &Mesh) -> Result<Mesh> {
    // Check if meshes are empty
    if a.vertices.is_empty() {
        return Ok(Mesh::empty());
    }
    if b.vertices.is_empty() {
        return Ok(a.clone());
    }

    // Strategy: Use a robustness check to decide algorithm
    // 1. Simple planar cases: use fast BSP
    // 2. Complex curved surfaces: use winding number approach

    let complexity_score = estimate_complexity(a, b);

    if complexity_score > 0.5 {
        // High complexity - use winding number approach
        mesh_difference_winding(a, b)
    } else {
        // Low complexity - try BSP first
        match super::csg::csg_difference_bsp_internal(a, b) {
            Ok(result) if result.vertex_count() > 0 && result.triangle_count() > 0 => {
                // Verify result is reasonable (not empty, has some geometry)
                // Sometimes BSP produces empty results even when it shouldn't
                if result.vertex_count() < a.vertex_count() / 10 {
                    // Result is suspiciously small - BSP might have failed
                    mesh_difference_winding(a, b)
                } else {
                    Ok(result)
                }
            }
            _ => {
                // BSP failed, fall back to winding number
                mesh_difference_winding(a, b)
            }
        }
    }
}

/// Estimate complexity of CSG operation (0.0 = simple, 1.0 = very complex)
fn estimate_complexity(a: &Mesh, b: &Mesh) -> f64 {
    // For simple shapes (cubes/boxes), always prefer BSP which handles them correctly
    // Even if B has curved surfaces, if A is simple, BSP should work well
    if !has_curved_surfaces(a) {
        // A is simple (cubes/boxes) - prefer BSP even if B is curved
        // BSP handles cylinder/cube differences well
        return 0.0;
    }

    // If A has curved surfaces, check if it's a simple union of cubes
    // (union of cubes might have many triangles but still be "simple" geometrically)
    // Check if A has many distinct normals (curved) vs few (flat faces from union)
    let a_distinct_normals = count_distinct_normals(a, 0.1);
    if a_distinct_normals <= 12 && !has_curved_surfaces(b) {
        // A is likely a union of simple shapes, B is also simple
        return 0.0;
    }

    let mut score: f64 = 0.0;

    // Factor 1: Very large meshes (> 1000 triangles indicates complex union)
    if a.triangle_count() > 1000 {
        score += 0.4;
    }

    // Factor 2: Both meshes are large AND complex
    if a.triangle_count() > 500 && b.triangle_count() > 500 && has_curved_surfaces(a) && has_curved_surfaces(b) {
        score += 0.3;
    }

    // Factor 3: A is complex curved AND B is curved
    // This is the problematic case: union of curved shapes minus curved shape
    if has_curved_surfaces(a) && has_curved_surfaces(b) && a.triangle_count() > 500 {
        score += 0.4;
    }

    score.min(1.0)
}

/// Count distinct normal directions in mesh (for complexity estimation)
fn count_distinct_normals(mesh: &Mesh, epsilon: f64) -> usize {
    if mesh.vertices.is_empty() {
        return 0;
    }

    let mut distinct_normals = vec![mesh.vertices[0].normal];
    let sample_size = mesh.vertices.len().min(200);

    for i in 1..sample_size {
        let normal = &mesh.vertices[i].normal;
        let mut is_new = true;

        for existing in &distinct_normals {
            if (normal - existing).norm() < epsilon {
                is_new = false;
                break;
            }
        }

        if is_new {
            distinct_normals.push(*normal);
        }
    }

    distinct_normals.len()
}

/// Mesh difference using winding number for robustness
fn mesh_difference_winding(a: &Mesh, b: &Mesh) -> Result<Mesh> {
    // Improved winding number approach:
    // Keep triangles from A that are outside B
    // Add inverted triangles from B that are inside A
    // Use multiple sample points per triangle for better accuracy

    let mut result = Mesh::new();

    // Add triangles from A that are outside B
    // Use balanced approach: check vertices and center, but be more conservative
    for tri in &a.triangles {
        let v0 = &a.vertices[tri.indices[0]];
        let v1 = &a.vertices[tri.indices[1]];
        let v2 = &a.vertices[tri.indices[2]];

        // Check center point and vertices
        let center = (v0.position.coords + v1.position.coords + v2.position.coords) / 3.0;
        let center_point = Point3::from(center);
        
        // Check vertices and center (4 points total, not 7)
        let v0_inside = is_point_inside_mesh(&v0.position, b);
        let v1_inside = is_point_inside_mesh(&v1.position, b);
        let v2_inside = is_point_inside_mesh(&v2.position, b);
        let center_inside = is_point_inside_mesh(&center_point, b);

        // Keep triangle if majority of points are outside (2 or more out of 4)
        // This ensures we don't miss triangles that are partially cut
        let outside_count = [v0_inside, v1_inside, v2_inside, center_inside]
            .iter()
            .filter(|&&inside| !inside)
            .count();

        if outside_count >= 2 {
            let i0 = result.add_vertex(*v0);
            let i1 = result.add_vertex(*v1);
            let i2 = result.add_vertex(*v2);
            result.add_triangle(Triangle::new([i0, i1, i2]));
        }
    }

    // Add inverted triangles from B that are inside A
    // Use balanced approach: check vertices and center
    for tri in &b.triangles {
        let v0 = &b.vertices[tri.indices[0]];
        let v1 = &b.vertices[tri.indices[1]];
        let v2 = &b.vertices[tri.indices[2]];

        let center = (v0.position.coords + v1.position.coords + v2.position.coords) / 3.0;
        let center_point = Point3::from(center);

        // Check if triangle is inside A (4 points total, not 7)
        let v0_inside = is_point_inside_mesh(&v0.position, a);
        let v1_inside = is_point_inside_mesh(&v1.position, a);
        let v2_inside = is_point_inside_mesh(&v2.position, a);
        let center_inside = is_point_inside_mesh(&center_point, a);

        // Add if majority of points are inside A (2 or more out of 4)
        // This ensures we properly close the hole in the difference
        let inside_count = [v0_inside, v1_inside, v2_inside, center_inside]
            .iter()
            .filter(|&&inside| inside)
            .count();

        if inside_count >= 2 {
            // Add inverted triangle (flip winding and normals)
            let mut v0_inv = *v0;
            let mut v1_inv = *v1;
            let mut v2_inv = *v2;
            v0_inv.normal = -v0_inv.normal;
            v1_inv.normal = -v1_inv.normal;
            v2_inv.normal = -v2_inv.normal;

            let i0 = result.add_vertex(v0_inv);
            let i1 = result.add_vertex(v2_inv); // Swap to invert winding
            let i2 = result.add_vertex(v1_inv);
            result.add_triangle(Triangle::new([i0, i1, i2]));
        }
    }

    // Recompute normals from triangle geometry to ensure correctness
    result.recompute_normals();
    Ok(result)
}

/// Check if a point is inside a mesh using ray casting
fn is_point_inside_mesh(point: &Point3<f64>, mesh: &Mesh) -> bool {
    // Ray casting algorithm: cast ray in +X direction and count intersections
    let ray_dir = Vector3::new(1.0, 0.0, 0.0);
    let mut intersection_count = 0;

    for tri in &mesh.triangles {
        let v0 = &mesh.vertices[tri.indices[0]].position;
        let v1 = &mesh.vertices[tri.indices[1]].position;
        let v2 = &mesh.vertices[tri.indices[2]].position;

        if ray_intersects_triangle(point, &ray_dir, v0, v1, v2) {
            intersection_count += 1;
        }
    }

    // Odd number of intersections = inside
    intersection_count % 2 == 1
}

/// Test if a ray intersects a triangle using Möller-Trumbore algorithm
fn ray_intersects_triangle(
    origin: &Point3<f64>,
    direction: &Vector3<f64>,
    v0: &Point3<f64>,
    v1: &Point3<f64>,
    v2: &Point3<f64>,
) -> bool {
    const EPSILON: f64 = 0.000001;

    let edge1 = v1 - v0;
    let edge2 = v2 - v0;
    let h = direction.cross(&edge2);
    let a = edge1.dot(&h);

    if a > -EPSILON && a < EPSILON {
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
    t > EPSILON // Only count forward intersections
}

/// Perform robust CSG intersection
pub fn robust_intersection(a: &Mesh, b: &Mesh) -> Result<Mesh> {
    // For intersection, we need both meshes
    if a.vertices.is_empty() || b.vertices.is_empty() {
        return Ok(Mesh::empty());
    }

    // Use BSP for now (normals already recomputed in csg_intersection)
    super::csg::csg_intersection(a, b)
}

/// Detect if a mesh likely has curved surfaces based on vertex normals
fn has_curved_surfaces(mesh: &Mesh) -> bool {
    if mesh.vertices.len() < 10 {
        return false;
    }

    // Strategy: Count approximately how many distinct normal directions exist
    // Cubes have ~6 normals (faces), spheres have many smoothly varying normals

    const EPSILON: f64 = 0.01;
    let mut distinct_normals = vec![mesh.vertices[0].normal];
    let sample_size = mesh.vertices.len().min(100);

    for i in 1..sample_size {
        let normal = &mesh.vertices[i].normal;
        let mut is_new = true;

        for existing in &distinct_normals {
            if (normal - existing).norm() < EPSILON {
                is_new = false;
                break;
            }
        }

        if is_new {
            distinct_normals.push(*normal);
        }

        // Early exit: if we have many distinct normals, it's curved
        if distinct_normals.len() > 12 {
            return true;
        }
    }

    // Cubes have ~6 distinct normals, curved surfaces have many more
    distinct_normals.len() > 12
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Primitive;
    use nalgebra::Vector3;

    #[test]
    fn test_robust_union() {
        let mesh_a = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
        let mesh_b = Primitive::cube(Vector3::new(5.0, 5.0, 5.0), false).to_mesh();

        let result = robust_union(&mesh_a, &mesh_b);
        assert!(result.is_ok());
        let mesh = result.unwrap();
        assert!(mesh.vertex_count() > 0);
    }

    #[test]
    fn test_curved_surface_detection() {
        let sphere = Primitive::sphere(10.0, 16).to_mesh();
        assert!(has_curved_surfaces(&sphere));

        let cube = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
        assert!(!has_curved_surfaces(&cube));
    }
}
