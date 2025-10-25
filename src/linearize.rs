//! Curve linearization - converts Bezier curves to line segments
//! 
//! This implementation uses adaptive subdivision based on curve angle,
//! matching the approach used by ttf2mesh for optimal performance.

use crate::error::Result;
use crate::types::{Contour, Outline2D, Point2D, Quality};
use std::f32::consts::PI;

const EPSILON: f32 = 1e-5;
const AREA_THRESHOLD: f32 = 1e-5;

/// Linearize an outline by converting curves to line segments
#[inline]
pub fn linearize_outline(outline: Outline2D, quality: Quality) -> Result<Outline2D> {
    let quality_value = quality.value();
    let mut result = Outline2D::new();

    for contour in outline.contours {
        let linearized = linearize_contour(&contour, quality_value);
        if !linearized.is_empty() {
            result.add_contour(linearized);
        }
    }

    Ok(result)
}

/// Linearize a single contour using adaptive subdivision
#[inline]
fn linearize_contour(contour: &Contour, quality: u8) -> Contour {
    let n = contour.points.len();
    if n < 2 {
        return contour.clone();
    }

    // Pre-allocate with estimate: most points stay + some subdivisions
    let estimated_size = n + (n / 3) * quality as usize;
    let mut result = Contour::new(contour.closed);
    result.points.reserve(estimated_size);

    let first_point = contour.points[0].point;

    // State machine matching ttf2mesh implementation
    // State 0: Initial/uninitialized
    // State 1: Have on-curve point, expecting next point
    // State 2: Have on-curve + off-curve, expecting end point or another off-curve
    let mut state = 0;
    let mut p0 = Point2D::ZERO; // Last on-curve point
    let mut p1 = Point2D::ZERO; // Control point (off-curve)

    for i in 0..n {
        let cp = contour.points[i];

        match state {
            0 => {
                // Initial state - add first point
                result.push_on_curve(cp.point);
                p0 = cp.point;
                state = 1;
            }
            1 => {
                // Have on-curve point, looking for next
                if cp.on_curve {
                    // Another on-curve point - add it
                    result.push_on_curve(cp.point);
                    p0 = cp.point;
                    state = 1;
                } else {
                    // Off-curve point - store as control point
                    p1 = cp.point;
                    state = 2;
                }
            }
            2 => {
                // Have on-curve + off-curve, expecting end point
                if cp.on_curve {
                    // Standard curve: on-off-on
                    linearize_qbezier(p0, p1, cp.point, quality, &mut result);
                    result.push_on_curve(cp.point);
                    p0 = cp.point;
                    state = 1;
                } else {
                    // Two consecutive off-curve points: on-off-off
                    // Insert implicit midpoint
                    let mid = (p1 + cp.point) * 0.5;
                    linearize_qbezier(p0, p1, mid, quality, &mut result);
                    result.push_on_curve(mid);
                    p0 = mid;
                    p1 = cp.point;
                    state = 2; // Stay in state 2, process current off-curve as next control
                }
            }
            _ => unreachable!(),
        }
    }

    // Handle closing curve if we ended with off-curve point
    if state == 2 && contour.closed {
        linearize_qbezier(p0, p1, first_point, quality, &mut result);
    }

    // Remove collinear points to reduce vertex count
    remove_collinear_points(&mut result);

    result
}

/// Remove near-collinear points from a contour (matches ttf_fix_linear_bags)
#[inline]
fn remove_collinear_points(contour: &mut Contour) {
    let n = contour.points.len();
    if n < 3 {
        return;
    }

    // Filter out collinear points by checking triangle area
    let mut kept = Vec::with_capacity(n);
    kept.push(contour.points[0]); // Always keep first point

    for i in 1..n - 1 {
        let p0 = contour.points[kept.len() - 1].point;
        let p1 = contour.points[i].point;
        let p2 = contour.points[i + 1].point;

        // Keep point if it forms a non-degenerate triangle
        if triangle_area(p0, p1, p2) > EPSILON {
            kept.push(contour.points[i]);
        }
    }

    // Always keep last point
    kept.push(contour.points[n - 1]);

    // Remove duplicate first/last points if they're too close
    while kept.len() > 1 {
        let first = kept[0].point;
        let last = kept[kept.len() - 1].point;
        let diff = last - first;
        if diff.x.abs() > EPSILON || diff.y.abs() > EPSILON {
            break;
        }
        kept.pop();
    }

    // Only update if we have enough points left
    if kept.len() >= 3 {
        contour.points = kept;
    }
}

/// Linearize a quadratic Bezier curve using adaptive subdivision
///
/// This matches the ttf2mesh approach: calculate the angle between tangents
/// at t=0 and t=1, then determine the number of subdivisions based on that angle.
#[inline(always)]
fn linearize_qbezier(p0: Point2D, p1: Point2D, p2: Point2D, quality: u8, result: &mut Contour) {
    // Check if the curve is nearly linear using triangle area (Heron's formula)
    let area = triangle_area(p0, p1, p2);
    if area < AREA_THRESHOLD {
        return; // Skip near-linear curves
    }

    // Calculate tangent vectors at t=0 and t=1 (inlined for performance)
    // At t=0: 2(P1-P0)
    let t0 = (p1 - p0) * 2.0;
    // At t=1: 2(P2-P1)
    let t1 = (p2 - p1) * 2.0;

    let t0_len_sq = t0.length_squared();
    let t1_len_sq = t1.length_squared();

    if t0_len_sq < EPSILON * EPSILON || t1_len_sq < EPSILON * EPSILON {
        return;
    }

    // Calculate angle between tangents using cross product
    // Use fast inverse sqrt to avoid two sqrt calls
    let cross = t0.x * t1.y - t0.y * t1.x;
    let inv_len_product = 1.0 / (t0_len_sq * t1_len_sq).sqrt();
    let mut angle = (cross.abs() * inv_len_product).min(1.0);

    // Convert to angle
    angle = angle.asin();

    // Calculate number of subdivisions based on angle
    let num_points = (angle / (PI * 2.0) * quality as f32).round() as usize;

    if num_points == 0 {
        return;
    }

    // Generate intermediate points
    let step = 1.0 / (num_points + 1) as f32;
    for i in 1..=num_points {
        let t = step * i as f32;
        let point = qbezier(p0, p1, p2, t);
        result.push_on_curve(point);
    }
}

/// Evaluate a quadratic Bezier curve at parameter t
#[inline(always)]
fn qbezier(p0: Point2D, p1: Point2D, p2: Point2D, t: f32) -> Point2D {
    // Optimized: reduce multiplications
    let one_minus_t = 1.0 - t;
    let b = one_minus_t * t;
    // a = (1-t)^2, c = t^2
    p0 * (one_minus_t * one_minus_t) + p1 * (2.0 * b) + p2 * (t * t)
}

/// Calculate triangle area using Heron's formula
#[inline(always)]
fn triangle_area(p0: Point2D, p1: Point2D, p2: Point2D) -> f32 {
    // Use length_squared to avoid sqrt until the end
    let a_sq = (p0 - p1).length_squared();
    let b_sq = (p1 - p2).length_squared();
    let c_sq = (p2 - p0).length_squared();

    // Fast path for very small triangles
    if a_sq < EPSILON * EPSILON || b_sq < EPSILON * EPSILON || c_sq < EPSILON * EPSILON {
        return 0.0;
    }

    let a = a_sq.sqrt();
    let b = b_sq.sqrt();
    let c = c_sq.sqrt();
    let s = (a + b + c) * 0.5;
    let area_sq = s * (s - a) * (s - b) * (s - c);

    if area_sq > 0.0 {
        area_sq.sqrt()
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    #[test]
    fn test_qbezier() {
        let p0 = Vec2::new(0.0, 0.0);
        let p1 = Vec2::new(0.5, 1.0);
        let p2 = Vec2::new(1.0, 0.0);

        let result = qbezier(p0, p1, p2, 0.0);
        assert!((result - p0).length() < 0.001);

        let result = qbezier(p0, p1, p2, 1.0);
        assert!((result - p2).length() < 0.001);

        let result = qbezier(p0, p1, p2, 0.5);
        assert!(result.y > 0.0);
    }
}
