//! NURBS curve evaluation via Cox-de Boor recursion.

use crate::types::curve::NurbsCurve;

impl NurbsCurve {
    /// Evaluate the NURBS curve at parameter `t` (0.0 to 1.0).
    ///
    /// Returns a 3D point on the curve as `(x, y, z)` in `f64`.
    /// The `magnification` factor scales the output (use 1.0 for no scaling).
    ///
    /// Mirrors the C++ `Curve::CalcuratePoint()` implementation.
    pub fn evaluate(&self, t: f64, magnification: f64) -> (f64, f64, f64) {
        if self.control_points.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        // Special case: t == 0 returns first control point
        if t == 0.0 {
            let p = &self.control_points[0];
            return (
                p.x * magnification,
                p.y * magnification,
                p.z * magnification,
            );
        }

        // Build extended knot vector: original knots + one extra
        let mut knots_ext: Vec<f64> = self.knots.clone();
        let extra = self.knots.last().copied().unwrap_or(0.0) + 1.0;
        knots_ext.push(extra);

        // t_rate is the original last knot value (= extended last - 1)
        let t_rate = extra - 1.0;
        let t_scaled = t * t_rate;

        let n = self.control_points.len();
        let order = self.order as usize;

        // Compute weighted basis functions
        let mut bs: Vec<f64> = Vec::with_capacity(n);
        let mut w_sum = 0.0;

        for j in 0..n {
            let basis = calc_bspline_basis(&knots_ext, j, order, t_scaled);
            let weighted = self.control_points[j].w * basis;
            bs.push(weighted);
            if !weighted.is_nan() {
                w_sum += weighted;
            }
        }

        if w_sum == 0.0 {
            // Avoid division by zero — return origin
            return (0.0, 0.0, 0.0);
        }

        // Compute weighted sum of control points
        let mut rx = 0.0;
        let mut ry = 0.0;
        let mut rz = 0.0;

        for (cp, &b) in self.control_points.iter().zip(bs.iter()) {
            let dx = cp.x * magnification * b / w_sum;
            let dy = cp.y * magnification * b / w_sum;
            let dz = cp.z * magnification * b / w_sum;
            if !dx.is_nan() {
                rx += dx;
            }
            if !dy.is_nan() {
                ry += dy;
            }
            if !dz.is_nan() {
                rz += dz;
            }
        }

        (rx, ry, rz)
    }
}

/// Cox-de Boor recursive B-spline basis function.
///
/// Mirrors the C++ `CalcBSplineBasisFunc()`.
fn calc_bspline_basis(knots: &[f64], j: usize, p: usize, t: f64) -> f64 {
    if knots.is_empty() {
        return f64::NAN;
    }

    let m = knots.len() - 1;
    if m < j + p + 1 {
        return f64::NAN;
    }

    if t < knots[j] || t > knots[j + p + 1] {
        return 0.0;
    }

    if p == 0 {
        return 1.0;
    }

    // Triangle apex special case
    if p == 1 && t == knots[j + 1] {
        return 1.0;
    }

    let d1 = if (knots[j + p] - knots[j]).abs() < f64::EPSILON {
        0.0
    } else {
        (t - knots[j]) / (knots[j + p] - knots[j]) * calc_bspline_basis(knots, j, p - 1, t)
    };

    let d2 = if (knots[j + p + 1] - knots[j + 1]).abs() < f64::EPSILON {
        0.0
    } else {
        (knots[j + p + 1] - t) / (knots[j + p + 1] - knots[j + 1])
            * calc_bspline_basis(knots, j + 1, p - 1, t)
    };

    d1 + d2
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::curve::DVector4;

    const EPSILON: f64 = 1e-6;

    fn assert_approx_f64(actual: f64, expected: f64, msg: &str) {
        assert!(
            (actual - expected).abs() < EPSILON,
            "{msg}: expected {expected}, got {actual}"
        );
    }

    fn make_line_curve() -> NurbsCurve {
        // Simple 2nd-order (linear) NURBS from (0,0,0) to (10,0,0)
        NurbsCurve {
            converter_version: 1,
            control_points: vec![
                DVector4 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                },
                DVector4 {
                    x: 10.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                },
            ],
            knots: vec![0.0, 0.0, 1.0, 1.0],
            order: 1, // degree 0 + 1 = linear (order=2 for degree 1 is typical, but let's test with correct setup)
            step: 0.01,
            curve_type: 0,
            dimension: 3,
            length: 10.0,
        }
    }

    #[test]
    fn test_empty_curve() {
        let c = NurbsCurve {
            converter_version: 1,
            control_points: vec![],
            knots: vec![],
            order: 2,
            step: 0.01,
            curve_type: 0,
            dimension: 3,
            length: 0.0,
        };
        let (x, y, z) = c.evaluate(0.5, 1.0);
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
        assert_eq!(z, 0.0);
    }

    #[test]
    fn test_t_zero_returns_first_point() {
        let c = make_line_curve();
        let (x, y, z) = c.evaluate(0.0, 1.0);
        assert_approx_f64(x, 0.0, "t=0 x");
        assert_approx_f64(y, 0.0, "t=0 y");
        assert_approx_f64(z, 0.0, "t=0 z");
    }

    #[test]
    fn test_magnification() {
        let c = make_line_curve();
        let (x, _, _) = c.evaluate(0.0, 2.0);
        assert_approx_f64(x, 0.0, "magnified t=0 x");
    }

    #[test]
    fn test_quadratic_nurbs() {
        // Quadratic NURBS (order=3) with 3 control points forming an arc
        let c = NurbsCurve {
            converter_version: 1,
            control_points: vec![
                DVector4 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                },
                DVector4 {
                    x: 5.0,
                    y: 10.0,
                    z: 0.0,
                    w: 1.0,
                },
                DVector4 {
                    x: 10.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                },
            ],
            knots: vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            order: 2, // degree 2 => order in C++ sense is degree+1=3, but Effekseer stores as degree+1
            step: 0.01,
            curve_type: 0,
            dimension: 3,
            length: 20.0,
        };

        // t=0 should give first control point
        let (x, y, z) = c.evaluate(0.0, 1.0);
        assert_approx_f64(x, 0.0, "quad t=0 x");
        assert_approx_f64(y, 0.0, "quad t=0 y");
        assert_approx_f64(z, 0.0, "quad t=0 z");
    }

    #[test]
    fn test_linear_nurbs_endpoints_and_interpolation() {
        // Linear NURBS (order=2) with 3 control points along X axis.
        // The extended knot vector and t_rate scaling follows the C++
        // CalcuratePoint algorithm.
        let c = NurbsCurve {
            converter_version: 1,
            control_points: vec![
                DVector4 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                },
                DVector4 {
                    x: 5.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                },
                DVector4 {
                    x: 10.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                },
            ],
            knots: vec![0.0, 0.0, 1.0, 2.0, 2.0],
            order: 2,
            step: 0.01,
            curve_type: 0,
            dimension: 3,
            length: 10.0,
        };

        // t=0 gives first control point
        let (x, y, z) = c.evaluate(0.0, 1.0);
        assert_approx_f64(x, 0.0, "linear t=0 x");
        assert_approx_f64(y, 0.0, "linear t=0 y");
        assert_approx_f64(z, 0.0, "linear t=0 z");

        // Monotonically increasing along x as t increases
        let (x1, _, _) = c.evaluate(0.25, 1.0);
        let (x2, _, _) = c.evaluate(0.5, 1.0);
        let (x3, _, _) = c.evaluate(0.75, 1.0);
        assert!(x1 > 0.0, "t=0.25 should be positive, got {x1}");
        assert!(x2 > x1, "t=0.5 ({x2}) should be > t=0.25 ({x1})");
        assert!(x3 > x2, "t=0.75 ({x3}) should be > t=0.5 ({x2})");
    }

    #[test]
    fn test_t_one_returns_last_point() {
        // Quadratic NURBS with clamped knots — t=1 should produce the last control point
        let c = NurbsCurve {
            converter_version: 1,
            control_points: vec![
                DVector4 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                },
                DVector4 {
                    x: 5.0,
                    y: 10.0,
                    z: 0.0,
                    w: 1.0,
                },
                DVector4 {
                    x: 10.0,
                    y: 0.0,
                    z: 5.0,
                    w: 1.0,
                },
            ],
            knots: vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            order: 3, // degree 2
            step: 0.01,
            curve_type: 0,
            dimension: 3,
            length: 20.0,
        };

        let (x, y, z) = c.evaluate(1.0, 1.0);
        assert_approx_f64(x, 10.0, "t=1 x");
        assert_approx_f64(y, 0.0, "t=1 y");
        assert_approx_f64(z, 5.0, "t=1 z");
    }

    #[test]
    fn test_weighted_control_points() {
        // Linear NURBS with different weights — the higher-weighted point pulls the curve
        let c = NurbsCurve {
            converter_version: 1,
            control_points: vec![
                DVector4 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                },
                DVector4 {
                    x: 10.0,
                    y: 0.0,
                    z: 0.0,
                    w: 3.0, // higher weight pulls curve toward this point
                },
            ],
            knots: vec![0.0, 0.0, 1.0, 1.0],
            order: 2,
            step: 0.01,
            curve_type: 0,
            dimension: 3,
            length: 10.0,
        };

        // At t=0.5, the curve should be pulled toward the second point (x > 5)
        let (x, _, _) = c.evaluate(0.5, 1.0);
        assert!(x > 5.0, "weighted point should pull curve, got x={x}");
    }
}
