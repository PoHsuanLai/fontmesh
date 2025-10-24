//! Curve linearization - converts Bezier curves to line segments

use crate::error::Result;
use crate::types::{Contour, Outline2D, Point2D, Quality};
use glam::Vec2;

/// Linearize an outline by converting curves to line segments
///
/// This function processes each contour and subdivides any Bezier curves
/// into straight line segments based on the quality parameter.
///
/// # Arguments
/// * `outline` - The outline with potentially curved segments
/// * `quality` - The subdivision level (higher = more segments)
///
/// # Returns
/// A new outline with all curves converted to lines
pub fn linearize_outline(outline: Outline2D, quality: Quality) -> Result<Outline2D> {
    let subdivisions = quality.value() as usize;
    let mut result = Outline2D::new();

    for contour in outline.contours {
        let linearized = linearize_contour(&contour, subdivisions);
        result.add_contour(linearized);
    }

    Ok(result)
}

/// Linearize a single contour
fn linearize_contour(contour: &Contour, subdivisions: usize) -> Contour {
    if contour.points.len() < 2 {
        return contour.clone();
    }

    let mut result = Contour::new(contour.closed);

    let mut i = 0;
    while i < contour.points.len() {
        let p0 = contour.points[i];
        result.push(p0);

        // Check if we have enough points for a curve
        if i + 2 < contour.points.len() {
            let p1 = contour.points[i + 1];
            let p2 = contour.points[i + 2];

            // Simple heuristic: if the angle is sharp enough, treat as quadratic bezier
            // For now, we'll do basic subdivision of any multi-point segment
            if subdivisions > 1 {
                subdivide_segment(p0, p1, p2, subdivisions, &mut result);
                i += 3;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    result
}

/// Subdivide a curved segment using De Casteljau's algorithm for quadratic Bezier
fn subdivide_segment(p0: Point2D, p1: Point2D, p2: Point2D, subdivisions: usize, result: &mut Contour) {
    // Use De Casteljau's algorithm to subdivide the curve
    for i in 1..=subdivisions {
        let t = i as f32 / subdivisions as f32;
        let point = quadratic_bezier(p0, p1, p2, t);
        result.push(point);
    }
}

/// Evaluate a quadratic Bezier curve at parameter t
fn quadratic_bezier(p0: Vec2, p1: Vec2, p2: Vec2, t: f32) -> Vec2 {
    let one_minus_t = 1.0 - t;
    let a = one_minus_t * one_minus_t;
    let b = 2.0 * one_minus_t * t;
    let c = t * t;

    p0 * a + p1 * b + p2 * c
}

/// Evaluate a cubic Bezier curve at parameter t
#[allow(dead_code)]
fn cubic_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let one_minus_t = 1.0 - t;
    let a = one_minus_t * one_minus_t * one_minus_t;
    let b = 3.0 * one_minus_t * one_minus_t * t;
    let c = 3.0 * one_minus_t * t * t;
    let d = t * t * t;

    p0 * a + p1 * b + p2 * c + p3 * d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quadratic_bezier() {
        let p0 = Vec2::new(0.0, 0.0);
        let p1 = Vec2::new(0.5, 1.0);
        let p2 = Vec2::new(1.0, 0.0);

        // Test at t=0 (should be p0)
        let result = quadratic_bezier(p0, p1, p2, 0.0);
        assert!((result - p0).length() < 0.001);

        // Test at t=1 (should be p2)
        let result = quadratic_bezier(p0, p1, p2, 1.0);
        assert!((result - p2).length() < 0.001);

        // Test at t=0.5 (should be between)
        let result = quadratic_bezier(p0, p1, p2, 0.5);
        assert!(result.y > 0.0); // Should be above the baseline
    }

    #[test]
    fn test_cubic_bezier() {
        let p0 = Vec2::new(0.0, 0.0);
        let p1 = Vec2::new(0.33, 1.0);
        let p2 = Vec2::new(0.66, 1.0);
        let p3 = Vec2::new(1.0, 0.0);

        // Test at t=0 (should be p0)
        let result = cubic_bezier(p0, p1, p2, p3, 0.0);
        assert!((result - p0).length() < 0.001);

        // Test at t=1 (should be p3)
        let result = cubic_bezier(p0, p1, p2, p3, 1.0);
        assert!((result - p3).length() < 0.001);
    }
}
