// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.
//
//! Visual diff rendering for STL files
//! Renders STL files to PNG images and compares them pixel-by-pixel
//
use anyhow::{bail, Context, Result};
use image::{Rgb, RgbImage};
use nalgebra::{Rotation3, Vector2, Vector3};
use std::fs::File;
use std::path::Path;
use stl_io::read_stl;

const DEFAULT_WIDTH: u32 = 1024;
const DEFAULT_HEIGHT: u32 = 768;

/// Render an STL mesh to a PNG image using a lightweight orthographic renderer.
/// The renderer performs simple triangle rasterization with depth buffering so
/// that developers can visually spot-check OpenSCAD vs Polyframe outputs.
pub fn render_stl_to_png(stl_path: &Path, output_png: &Path) -> Result<()> {
    if !stl_path.exists() {
        bail!("STL file not found: {}", stl_path.display());
    }

    let mut file = File::open(stl_path)
        .with_context(|| format!("Failed to open STL file: {}", stl_path.display()))?;
    let stl =
        read_stl(&mut file).context("Failed to read STL contents during preview rendering")?;

    if stl.vertices.is_empty() || stl.faces.is_empty() {
        bail!(
            "STL {} does not contain any triangles to render",
            stl_path.display()
        );
    }

    let rotation = Rotation3::from_euler_angles(-0.9, 0.8, 0.3);
    let transformed: Vec<Vector3<f32>> = stl
        .vertices
        .iter()
        .map(|v| rotation * Vector3::new(v[0], v[1], v[2]))
        .collect();

    let bounds = BoundingBox::from_points(&transformed);
    let projected = project_vertices(&transformed, &bounds);
    let depths: Vec<f32> = transformed.iter().map(|p| -p.z).collect();

    let mut image =
        RgbImage::from_pixel(DEFAULT_WIDTH, DEFAULT_HEIGHT, Rgb([15, 18, 26])); // dark background
    let mut depth_buffer = vec![f32::NEG_INFINITY; (DEFAULT_WIDTH * DEFAULT_HEIGHT) as usize];
    let light_dir = Vector3::new(0.35, 0.55, 1.0).normalize();

    for face in &stl.faces {
        let indices = face.vertices;
        let i0 = indices[0] as usize;
        let i1 = indices[1] as usize;
        let i2 = indices[2] as usize;

        let v0 = transformed[i0];
        let v1 = transformed[i1];
        let v2 = transformed[i2];

        let normal = (v1 - v0).cross(&(v2 - v0));
        if normal.norm_squared() < 1e-6 {
            continue;
        }

        let intensity = normal
            .normalize()
            .dot(&light_dir)
            .abs()
            .max(0.05)
            .min(1.0);

        let pts = [projected[i0], projected[i1], projected[i2]];
        let tri_depths = [depths[i0], depths[i1], depths[i2]];

        rasterize_triangle(
            &mut image,
            &mut depth_buffer,
            pts,
            tri_depths,
            intensity,
        );
    }

    image
        .save(output_png)
        .with_context(|| format!("Failed to save PNG to {}", output_png.display()))?;

    Ok(())
}

/// Compare two PNG images and return pixel delta percentage
pub fn compare_images(png1: &Path, png2: &Path) -> Result<f32> {
    use image::GenericImageView;

    let img1 = image::open(png1)
        .context(format!("Failed to open image: {}", png1.display()))?;
    let img2 = image::open(png2)
        .context(format!("Failed to open image: {}", png2.display()))?;

    let (width1, height1) = img1.dimensions();
    let (width2, height2) = img2.dimensions();

    if width1 != width2 || height1 != height2 {
        return Ok(100.0); // 100% different if dimensions don't match
    }

    let mut diff_pixels = 0u64;
    let total_pixels = (width1 * height1) as u64;

    for y in 0..height1 {
        for x in 0..width1 {
            let pixel1 = img1.get_pixel(x, y);
            let pixel2 = img2.get_pixel(x, y);

            // Calculate RGB difference
            let r_diff = (pixel1[0] as i32 - pixel2[0] as i32).abs();
            let g_diff = (pixel1[1] as i32 - pixel2[1] as i32).abs();
            let b_diff = (pixel1[2] as i32 - pixel2[2] as i32).abs();

            // Consider pixels different if any channel differs by more than threshold
            if r_diff > 5 || g_diff > 5 || b_diff > 5 {
                diff_pixels += 1;
            }
        }
    }

    let delta_pct = (diff_pixels as f32 / total_pixels as f32) * 100.0;
    Ok(delta_pct)
}

/// Generate visual diff image showing differences between two STL renders
pub fn generate_diff_image(
    openscad_png: &Path,
    polyframe_png: &Path,
    output_diff: &Path,
) -> Result<f32> {
    use image::{GenericImageView, ImageBuffer, Rgb, RgbImage};

    let img1 = image::open(openscad_png)
        .context(format!("Failed to open image: {}", openscad_png.display()))?;
    let img2 = image::open(polyframe_png)
        .context(format!("Failed to open image: {}", polyframe_png.display()))?;

    let (width, height) = img1.dimensions();

    let mut diff_img: RgbImage = ImageBuffer::new(width, height);
    let mut diff_pixels = 0u64;
    let total_pixels = (width * height) as u64;

    for y in 0..height {
        for x in 0..width {
            let pixel1 = img1.get_pixel(x, y);
            let pixel2 = img2.get_pixel(x, y);

            let r_diff = (pixel1[0] as i32 - pixel2[0] as i32).abs() as u8;
            let g_diff = (pixel1[1] as i32 - pixel2[1] as i32).abs() as u8;
            let b_diff = (pixel1[2] as i32 - pixel2[2] as i32).abs() as u8;

            // Highlight differences in red
            if r_diff > 5 || g_diff > 5 || b_diff > 5 {
                diff_pixels += 1;
                diff_img.put_pixel(x, y, Rgb([255, 0, 0])); // Red for differences
            } else {
                // Grayscale for similarities
                let gray = ((pixel1[0] as u32 + pixel1[1] as u32 + pixel1[2] as u32) / 3) as u8;
                diff_img.put_pixel(x, y, Rgb([gray, gray, gray]));
            }
        }
    }

    diff_img
        .save(output_diff)
        .context(format!("Failed to save diff image to {}", output_diff.display()))?;

    let delta_pct = (diff_pixels as f32 / total_pixels as f32) * 100.0;
    Ok(delta_pct)
}

fn project_vertices(points: &[Vector3<f32>], bounds: &BoundingBox) -> Vec<Vector2<f32>> {
    if points.is_empty() {
        return Vec::new();
    }

    let span_x = (bounds.max_x - bounds.min_x).max(1e-3);
    let span_y = (bounds.max_y - bounds.min_y).max(1e-3);
    let scale = 0.9
        * (DEFAULT_WIDTH as f32 / span_x)
            .min(DEFAULT_HEIGHT as f32 / span_y)
            .max(1e-3);

    let x_offset = (DEFAULT_WIDTH as f32 - span_x * scale) * 0.5;
    let y_offset = (DEFAULT_HEIGHT as f32 - span_y * scale) * 0.5;

    points
        .iter()
        .map(|p| {
            Vector2::new(
                (p.x - bounds.min_x) * scale + x_offset,
                (bounds.max_y - p.y) * scale + y_offset,
            )
        })
        .collect()
}

fn rasterize_triangle(
    image: &mut RgbImage,
    depth_buffer: &mut [f32],
    points: [Vector2<f32>; 3],
    depths: [f32; 3],
    intensity: f32,
) {
    let width = image.width() as i32;
    let height = image.height() as i32;

    let min_x = points
        .iter()
        .fold(f32::INFINITY, |acc, p| acc.min(p.x))
        .floor()
        .max(0.0) as i32;
    let max_x = points
        .iter()
        .fold(f32::NEG_INFINITY, |acc, p| acc.max(p.x))
        .ceil()
        .min((width - 1) as f32) as i32;
    let min_y = points
        .iter()
        .fold(f32::INFINITY, |acc, p| acc.min(p.y))
        .floor()
        .max(0.0) as i32;
    let max_y = points
        .iter()
        .fold(f32::NEG_INFINITY, |acc, p| acc.max(p.y))
        .ceil()
        .min((height - 1) as f32) as i32;

    if min_x >= max_x || min_y >= max_y {
        return;
    }

    let area = edge(points[0], points[1], points[2]);
    if area.abs() < 1e-4 {
        return;
    }
    let inv_area = 1.0 / area;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let p = Vector2::new(x as f32 + 0.5, y as f32 + 0.5);
            let w0 = edge(points[1], points[2], p);
            let w1 = edge(points[2], points[0], p);
            let w2 = edge(points[0], points[1], p);

            let same_sign = (w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0)
                || (w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0);

            if !same_sign {
                continue;
            }

            let b0 = w0 * inv_area;
            let b1 = w1 * inv_area;
            let b2 = w2 * inv_area;
            let depth = b0 * depths[0] + b1 * depths[1] + b2 * depths[2];

            let idx = (y as u32 * image.width() + x as u32) as usize;
            if depth > depth_buffer[idx] {
                depth_buffer[idx] = depth;
                let shade = (intensity * 205.0 + 40.0).clamp(0.0, 255.0) as u8;
                let color = Rgb([
                    shade,
                    (shade as f32 * 0.92) as u8,
                    (shade as f32 * 0.78 + 20.0).min(255.0) as u8,
                ]);
                image.put_pixel(x as u32, y as u32, color);
            }
        }
    }
}

fn edge(a: Vector2<f32>, b: Vector2<f32>, p: Vector2<f32>) -> f32 {
    (p.x - a.x) * (b.y - a.y) - (p.y - a.y) * (b.x - a.x)
}

struct BoundingBox {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

impl BoundingBox {
    fn from_points(points: &[Vector3<f32>]) -> Self {
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for p in points {
            min_x = min_x.min(p.x);
            max_x = max_x.max(p.x);
            min_y = min_y.min(p.y);
            max_y = max_y.max(p.y);
        }

        Self {
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{geometry::Primitive, io};
    use nalgebra::Vector3;
    use tempfile::TempDir;

    #[test]
    fn test_render_and_compare_images() {
        let temp_dir = TempDir::new().unwrap();
        let stl_path = temp_dir.path().join("cube.stl");
        let mesh = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), true).to_mesh();
        io::export_stl(&mesh, stl_path.to_str().unwrap()).unwrap();

        let png1 = temp_dir.path().join("render1.png");
        let png2 = temp_dir.path().join("render2.png");

        render_stl_to_png(&stl_path, &png1).unwrap();
        render_stl_to_png(&stl_path, &png2).unwrap();

        let delta = compare_images(&png1, &png2).unwrap();
        assert!(delta < 1.0, "Expected nearly identical renders, got {}", delta);
    }
}

