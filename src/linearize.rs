//! Curve linearization — converts Bézier curves to line segments.
//!
//! Quadratic (TrueType) curves use adaptive subdivision based on the angle
//! between end tangents, matching ttf2mesh. Cubic (CFF) curves are sampled
//! uniformly; a later collinear-point pass drops over-sampled vertices.
//!
//! ```
//! use fontmesh::linearize_outline;
//! use fontmesh::types::{Contour, Outline2D, Point2D};
//!
//! let mut c = Contour::new(true);
//! c.push_on_curve(Point2D::new(0.0, 0.0));
//! c.push_off_curve(Point2D::new(0.5, 1.0));
//! c.push_on_curve(Point2D::new(1.0, 0.0));
//! let mut outline = Outline2D::new();
//! outline.add_contour(c);
//! let flat = linearize_outline(outline, 16).unwrap();
//! assert!(flat.contours[0].points.iter().all(|p| p.on_curve));
//! ```

use crate::error::Result;
use crate::types::{Contour, CurveKind, Outline2D, Point2D};
use std::f32::consts::PI;

const EPSILON: f32 = 1e-5;
const AREA_THRESHOLD: f32 = 1e-5;

/// Linearize an outline by converting Bézier curves to line segments.
///
/// Off-curve controls are sampled away; the returned contours contain only
/// on-curve points. `subdivisions` is the quality knob used by
/// [`crate::glyph_to_mesh_2d`].
///
/// # Arguments
/// * `outline` - The outline to linearize (consumed)
/// * `subdivisions` - Target samples per curve; quadratics may use fewer
///   when the arc is shallow
///
/// ```
/// use fontmesh::linearize_outline;
/// use fontmesh::types::{Contour, Outline2D, Point2D};
///
/// let mut c = Contour::new(true);
/// c.push_on_curve(Point2D::new(0.0, 0.0));
/// c.push_on_curve(Point2D::new(1.0, 0.0));
/// c.push_on_curve(Point2D::new(0.0, 1.0));
/// let mut outline = Outline2D::new();
/// outline.add_contour(c);
/// let flat = linearize_outline(outline, 8).unwrap();
/// assert_eq!(flat.contours.len(), 1);
/// ```
#[inline]
pub fn linearize_outline(outline: Outline2D, subdivisions: u8) -> Result<Outline2D> {
    let mut result = Outline2D::new();

    outline
        .contours
        .into_iter()
        .map(|contour| linearize_contour(&contour, subdivisions))
        .filter(|linearized| !linearized.is_empty())
        .for_each(|linearized| result.add_contour(linearized));

    Ok(result)
}

/// State machine for processing contour points.
///
/// Supports both TrueType-style quadratic chains (one off-curve between two
/// on-curve points, with implicit midpoints inserted when two off-curves are
/// adjacent) and CFF-style cubics (a `Cubic`-tagged pair of off-curve points
/// between two on-curve points).
#[derive(Debug, Clone, Copy)]
enum LinearizeState {
    /// Initial state - expecting first point
    Initial,
    /// Have an on-curve point, expecting next point
    OnCurve { last_point: Point2D },
    /// Have on-curve + one quadratic off-curve, expecting end point or another off-curve
    QuadCtrl {
        last_point: Point2D,
        control_point: Point2D,
    },
    /// Have on-curve + first cubic control point, expecting second cubic control
    CubicCtrl1 {
        last_point: Point2D,
        control_point: Point2D,
    },
    /// Have on-curve + both cubic control points, expecting end point
    CubicCtrl2 {
        last_point: Point2D,
        control_point_1: Point2D,
        control_point_2: Point2D,
    },
}

/// Linearize a single contour using adaptive subdivision
#[inline]
fn linearize_contour(contour: &Contour, subdivisions: u8) -> Contour {
    let n = contour.points.len();
    if n < 2 {
        let mut result = Contour::new(contour.closed);
        result.points = contour.points.clone();
        return result;
    }

    let estimated_size = n + (n / 3) * subdivisions as usize;
    let mut result = Contour::new(contour.closed);
    result.points.reserve(estimated_size);

    let first_point = contour.points[0].point;

    let mut state = LinearizeState::Initial;

    for i in 0..n {
        let cp = contour.points[i];

        state = match state {
            LinearizeState::Initial => {
                result.push_on_curve(cp.point);
                LinearizeState::OnCurve {
                    last_point: cp.point,
                }
            }
            LinearizeState::OnCurve { last_point } => {
                if cp.on_curve {
                    result.push_on_curve(cp.point);
                    LinearizeState::OnCurve {
                        last_point: cp.point,
                    }
                } else if cp.curve_kind == CurveKind::Cubic {
                    LinearizeState::CubicCtrl1 {
                        last_point,
                        control_point: cp.point,
                    }
                } else {
                    LinearizeState::QuadCtrl {
                        last_point,
                        control_point: cp.point,
                    }
                }
            }
            LinearizeState::QuadCtrl {
                last_point,
                control_point,
            } => {
                if cp.on_curve {
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
                    // Two consecutive off-curve points in a quad chain imply
                    // an on-curve midpoint between them (TrueType convention).
                    let mid = (control_point + cp.point) * 0.5;
                    linearize_qbezier(last_point, control_point, mid, subdivisions, &mut result);
                    result.push_on_curve(mid);
                    LinearizeState::QuadCtrl {
                        last_point: mid,
                        control_point: cp.point,
                    }
                }
            }
            LinearizeState::CubicCtrl1 {
                last_point,
                control_point,
            } => {
                if !cp.on_curve {
                    LinearizeState::CubicCtrl2 {
                        last_point,
                        control_point_1: control_point,
                        control_point_2: cp.point,
                    }
                } else {
                    // Malformed cubic chain; degrade to a quadratic so we
                    // don't lose the segment.
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
                }
            }
            LinearizeState::CubicCtrl2 {
                last_point,
                control_point_1,
                control_point_2,
            } => {
                linearize_cbezier(
                    last_point,
                    control_point_1,
                    control_point_2,
                    cp.point,
                    subdivisions,
                    &mut result,
                );
                result.push_on_curve(cp.point);
                LinearizeState::OnCurve {
                    last_point: cp.point,
                }
            }
        };
    }

    // Flush a dangling curve that ran off the end of a closed contour by
    // closing it back to the first point.
    match state {
        LinearizeState::QuadCtrl {
            last_point,
            control_point,
        } if contour.closed => {
            linearize_qbezier(
                last_point,
                control_point,
                first_point,
                subdivisions,
                &mut result,
            );
        }
        LinearizeState::CubicCtrl2 {
            last_point,
            control_point_1,
            control_point_2,
        } if contour.closed => {
            linearize_cbezier(
                last_point,
                control_point_1,
                control_point_2,
                first_point,
                subdivisions,
                &mut result,
            );
        }
        _ => {}
    }

    remove_collinear_points(&mut result);

    result
}

/// Remove near-collinear points from a contour (matches ttf_fix_linear_bags).
#[inline]
fn remove_collinear_points(contour: &mut Contour) {
    let n = contour.points.len();
    if n < 3 {
        return;
    }

    let mut write_idx = 1;

    for read_idx in 1..n - 1 {
        let p0 = contour.points[write_idx - 1].point;
        let p1 = contour.points[read_idx].point;
        let p2 = contour.points[read_idx + 1].point;

        if triangle_area(p0, p1, p2) > EPSILON {
            if write_idx != read_idx {
                contour.points[write_idx] = contour.points[read_idx];
            }
            write_idx += 1;
        }
    }

    if write_idx != n - 1 {
        contour.points[write_idx] = contour.points[n - 1];
    }
    write_idx += 1;

    contour.points.truncate(write_idx);

    while contour.points.len() > 1 {
        let first = contour.points[0].point;
        let last = contour.points[contour.points.len() - 1].point;
        let diff = last - first;
        if diff.x.abs() > EPSILON || diff.y.abs() > EPSILON {
            break;
        }
        contour.points.pop();
    }

    if contour.points.len() < 3 {
        contour.points.clear();
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
    let area = triangle_area(p0, p1, p2);
    if area < AREA_THRESHOLD {
        return;
    }

    // Tangent vectors at t=0 and t=1.
    let t0 = (p1 - p0) * 2.0;
    let t1 = (p2 - p1) * 2.0;

    let t0_len = t0.length();
    let t1_len = t1.length();

    if t0_len < EPSILON || t1_len < EPSILON {
        return;
    }

    let cross = t0.x * t1.y - t0.y * t1.x;
    let inv_len_product = 1.0 / (t0_len * t1_len);
    let mut angle = (cross.abs() * inv_len_product).min(1.0);
    angle = angle.asin();

    // Sample density scales with the angle swept by the curve.
    let num_points = (angle / (PI * 2.0) * subdivisions as f32).round() as usize;
    if num_points == 0 {
        return;
    }

    // Loop unrolled in batches of 4 to give the inner Bezier eval more ILP.
    let step = 1.0 / (num_points + 1) as f32;
    let batch_size = 4;
    let full_batches = num_points / batch_size;

    let mut t = step;
    for _ in 0..full_batches {
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

    (0..(num_points % batch_size)).fold(t, |t, _| {
        result.push_on_curve(qbezier(p0, p1, p2, t));
        t + step
    });
}

/// Evaluate a quadratic Bezier curve at parameter t.
#[inline(always)]
fn qbezier(p0: Point2D, p1: Point2D, p2: Point2D, t: f32) -> Point2D {
    let one_minus_t = 1.0 - t;
    let b = one_minus_t * t;
    p0 * (one_minus_t * one_minus_t) + p1 * (2.0 * b) + p2 * (t * t)
}

/// Linearize a cubic Bezier curve using uniform subdivision.
///
/// Cubics from CFF/PostScript fonts can curve more sharply than the
/// per-glyph average, so we don't try to be clever with adaptive angle
/// estimation here — we just sample uniformly across the curve. The
/// `remove_collinear_points` pass later prunes any over-sampling.
#[inline]
fn linearize_cbezier(
    p0: Point2D,
    p1: Point2D,
    p2: Point2D,
    p3: Point2D,
    subdivisions: u8,
    result: &mut Contour,
) {
    let n = subdivisions.max(1) as usize;
    let step = 1.0 / (n as f32 + 1.0);
    let mut t = step;
    for _ in 0..n {
        result.push_on_curve(cbezier(p0, p1, p2, p3, t));
        t += step;
    }
}

/// Evaluate a cubic Bezier curve at parameter t.
#[inline(always)]
fn cbezier(p0: Point2D, p1: Point2D, p2: Point2D, p3: Point2D, t: f32) -> Point2D {
    let omt = 1.0 - t;
    let omt2 = omt * omt;
    let t2 = t * t;
    p0 * (omt2 * omt) + p1 * (3.0 * omt2 * t) + p2 * (3.0 * omt * t2) + p3 * (t2 * t)
}

/// Calculate triangle area using Heron's formula.
#[inline(always)]
fn triangle_area(p0: Point2D, p1: Point2D, p2: Point2D) -> f32 {
    let a_sq = (p0 - p1).length_squared();
    let b_sq = (p1 - p2).length_squared();
    let c_sq = (p2 - p0).length_squared();

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
