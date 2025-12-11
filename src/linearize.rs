//! Curve linearization - converts Bezier curves to line segments
//!
//! This implementation uses adaptive subdivision based on curve angle,
//! matching the approach used by ttf2mesh for optimal performance.

use crate::error::Result;
use crate::types::{Contour, Outline2D, Point2D};
use std::f32::consts::PI;

const EPSILON: f32 = 1e-5;
const AREA_THRESHOLD: f32 = 1e-5;

/// Linearize an outline by converting curves to line segments
///
/// # Arguments
/// * `outline` - The outline to linearize
/// * `subdivisions` - Number of subdivisions per curve
#[inline]
pub fn linearize_outline(outline: Outline2D, subdivisions: u8) -> Result<Outline2D> {
    let mut result = Outline2D::new();

    for contour in outline.contours {
        let linearized = linearize_contour(&contour, subdivisions);
        if !linearized.is_empty() {
            result.add_contour(linearized);
        }
    }

    Ok(result)
}

/// State machine for processing TrueType contour points
#[derive(Debug, Clone, Copy)]
enum LinearizeState {
    /// Initial state - expecting first point
    Initial,
    /// Have an on-curve point, expecting next point
    OnCurve { last_point: Point2D },
    /// Have on-curve + off-curve, expecting end point or another off-curve
    OffCurve {
        last_point: Point2D,
        control_point: Point2D,
    },
}

/// Linearize a single contour using adaptive subdivision
#[inline]
fn linearize_contour(contour: &Contour, subdivisions: u8) -> Contour {
    let n = contour.points.len();
    if n < 2 {
        // Return a new contour with just the points (avoid cloning entire structure)
        let mut result = Contour::new(contour.closed);
        result.points = contour.points.clone();
        return result;
    }

    // Pre-allocate with estimate: most points stay + some subdivisions
    let estimated_size = n + (n / 3) * subdivisions as usize;
    let mut result = Contour::new(contour.closed);
    result.points.reserve(estimated_size);

    let first_point = contour.points[0].point;

    // State machine for processing TrueType contour points
    let mut state = LinearizeState::Initial;

    for i in 0..n {
        let cp = contour.points[i];

        state = match state {
            LinearizeState::Initial => {
                // Initial state - add first point
                result.push_on_curve(cp.point);
                LinearizeState::OnCurve {
                    last_point: cp.point,
                }
            }
            LinearizeState::OnCurve { last_point } => {
                // Have on-curve point, looking for next
                if cp.on_curve {
                    // Another on-curve point - add it
                    result.push_on_curve(cp.point);
                    LinearizeState::OnCurve {
                        last_point: cp.point,
                    }
                } else {
                    // Off-curve point - store as control point
                    LinearizeState::OffCurve {
                        last_point,
                        control_point: cp.point,
                    }
                }
            }
            LinearizeState::OffCurve {
                last_point,
                control_point,
            } => {
                // Have on-curve + off-curve, expecting end point
                if cp.on_curve {
                    // Standard curve: on-off-on
                    linearize_qbezier(
                        last_point,
                        control_point,
                        cp.point,
                        subdivisions,
                        &mut result,
                    );
                    result.push_on_curve(cp.point);
                    LinearizeState::OnCurve {
                        last_point: cp.point,
                    }
                } else {
                    // Two consecutive off-curve points: on-off-off
                    // Insert implicit midpoint
                    let mid = (control_point + cp.point) * 0.5;
                    linearize_qbezier(last_point, control_point, mid, subdivisions, &mut result);
                    result.push_on_curve(mid);
                    LinearizeState::OffCurve {
                        last_point: mid,
                        control_point: cp.point,
                    }
                }
            }
        };
    }

    // Handle closing curve if we ended with off-curve point
    if let LinearizeState::OffCurve {
        last_point,
        control_point,
    } = state
    {
        if contour.closed {
            linearize_qbezier(
                last_point,
                control_point,
                first_point,
                subdivisions,
                &mut result,
            );
        }
    }

    // Remove collinear points to reduce vertex count
    remove_collinear_points(&mut result);

    result
}

/// Remove near-collinear points from a contour (matches ttf_fix_linear_bags)
/// Optimized: uses in-place two-pointer algorithm to avoid allocations
#[inline]
fn remove_collinear_points(contour: &mut Contour) {
    let n = contour.points.len();
    if n < 3 {
        return;
    }

    // In-place two-pointer algorithm
    // write_idx tracks where to write the next kept point
    let mut write_idx = 1; // Start after first point (always kept)

    for read_idx in 1..n - 1 {
        // Check if the point at read_idx should be kept
        let p0 = contour.points[write_idx - 1].point;
        let p1 = contour.points[read_idx].point;
        let p2 = contour.points[read_idx + 1].point;

        // Keep point if it forms a non-degenerate triangle
        if triangle_area(p0, p1, p2) > EPSILON {
            if write_idx != read_idx {
                contour.points[write_idx] = contour.points[read_idx];
            }
            write_idx += 1;
        }
    }

    // Always keep last point
    if write_idx != n - 1 {
        contour.points[write_idx] = contour.points[n - 1];
    }
    write_idx += 1;

    // Truncate to the number of kept points
    contour.points.truncate(write_idx);

    // Remove duplicate first/last points if they're too close
    while contour.points.len() > 1 {
        let first = contour.points[0].point;
        let last = contour.points[contour.points.len() - 1].point;
        let diff = last - first;
        if diff.x.abs() > EPSILON || diff.y.abs() > EPSILON {
            break;
        }
        contour.points.pop();
    }

    // If we have fewer than 3 points left, restore to a minimal valid state
    if contour.points.len() < 3 {
        // This shouldn't happen in normal cases, but be defensive
        contour.points.truncate(0);
    }
}

/// Linearize a quadratic Bezier curve using adaptive subdivision
///
/// This matches the ttf2mesh approach: calculate the angle between tangents
/// at t=0 and t=1, then determine the number of subdivisions based on that angle.
#[inline(always)]
fn linearize_qbezier(
    p0: Point2D,
    p1: Point2D,
    p2: Point2D,
    subdivisions: u8,
    result: &mut Contour,
) {
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

    let t0_len = t0.length();
    let t1_len = t1.length();

    if t0_len < EPSILON || t1_len < EPSILON {
        return;
    }

    // Calculate angle between tangents using cross product
    let cross = t0.x * t1.y - t0.y * t1.x;
    let inv_len_product = 1.0 / (t0_len * t1_len);
    let mut angle = (cross.abs() * inv_len_product).min(1.0);

    // Convert to angle
    angle = angle.asin();

    // Calculate number of subdivisions based on angle
    let num_points = (angle / (PI * 2.0) * subdivisions as f32).round() as usize;

    if num_points == 0 {
        return;
    }

    // Generate intermediate points
    // Optimized: batch process 4 points at a time for better CPU utilization
    let step = 1.0 / (num_points + 1) as f32;

    // Process in batches of 4
    let batch_size = 4;
    let full_batches = num_points / batch_size;

    let mut t = step;
    for _ in 0..full_batches {
        // Compute 4 points in sequence (compiler can optimize this better)
        let t0 = t;
        let t1 = t + step;
        let t2 = t + step * 2.0;
        let t3 = t + step * 3.0;

        result.push_on_curve(qbezier(p0, p1, p2, t0));
        result.push_on_curve(qbezier(p0, p1, p2, t1));
        result.push_on_curve(qbezier(p0, p1, p2, t2));
        result.push_on_curve(qbezier(p0, p1, p2, t3));

        t += step * 4.0;
    }

    // Handle remaining points
    for _ in 0..(num_points % batch_size) {
        result.push_on_curve(qbezier(p0, p1, p2, t));
        t += step;
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
