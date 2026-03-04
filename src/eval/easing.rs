//! Easing function evaluation for the Easing3Type system.

use crate::types::enums::Easing3Type;

/// Evaluate the bounce-out helper used by all three bounce easings.
fn ease_out_bounce(t: f32) -> f32 {
    const N1: f32 = 7.5625;
    if t < 4.0 / 11.0 {
        N1 * t * t
    } else if t < 8.0 / 11.0 {
        let t2 = t - 6.0 / 11.0;
        N1 * t2 * t2 + 0.75
    } else if t < 10.0 / 11.0 {
        let t2 = t - 9.0 / 11.0;
        N1 * t2 * t2 + 0.9375
    } else {
        let t2 = t - 21.0 / 22.0;
        N1 * t2 * t2 + 0.984375
    }
}

/// Evaluate an easing function.
///
/// `t` is clamped to \[0, 1\] before evaluation.
/// For `StartEndSpeed`, the stored `(a, b, c)` parameters define the cubic
/// polynomial `a*t³ + b*t² + c*t`. For all other easing types, `a`, `b`, and
/// `c` are ignored.
///
/// Returns a value in approximately \[0, 1\] — Back and Bounce easings may
/// overshoot.
pub fn evaluate_easing(easing_type: Easing3Type, t: f32, a: f32, b: f32, c: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match easing_type {
        // Cubic speed curve using stored parameters
        Easing3Type::StartEndSpeed => ((a * t + b) * t + c) * t,

        // Linear
        Easing3Type::Linear => t,

        // Quadratic
        Easing3Type::EaseInQuadratic => t * t,
        Easing3Type::EaseOutQuadratic => 1.0 - (1.0 - t).powi(2),
        Easing3Type::EaseInOutQuadratic => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
            }
        }

        // Cubic
        Easing3Type::EaseInCubic => t * t * t,
        Easing3Type::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
        Easing3Type::EaseInOutCubic => {
            if t < 0.5 {
                4.0 * t * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
            }
        }

        // Quartic
        Easing3Type::EaseInQuartic => t.powi(4),
        Easing3Type::EaseOutQuartic => 1.0 - (1.0 - t).powi(4),
        Easing3Type::EaseInOutQuartic => {
            if t < 0.5 {
                8.0 * t.powi(4)
            } else {
                1.0 - (-2.0 * t + 2.0).powi(4) / 2.0
            }
        }

        // Quintic
        Easing3Type::EaseInQuintic => t.powi(5),
        Easing3Type::EaseOutQuintic => 1.0 - (1.0 - t).powi(5),
        Easing3Type::EaseInOutQuintic => {
            if t < 0.5 {
                16.0 * t.powi(5)
            } else {
                1.0 - (-2.0 * t + 2.0).powi(5) / 2.0
            }
        }

        // Back (constant c1 = 1.8)
        Easing3Type::EaseInBack => {
            const C1: f32 = 1.8;
            const C3: f32 = C1 + 1.0;
            C3 * t * t * t - C1 * t * t
        }
        Easing3Type::EaseOutBack => {
            const C1: f32 = 1.8;
            const C3: f32 = C1 + 1.0;
            1.0 + C3 * (t - 1.0).powi(3) + C1 * (t - 1.0).powi(2)
        }
        Easing3Type::EaseInOutBack => {
            const C1: f32 = 1.8;
            const C2: f32 = C1 * 1.525;
            if t < 0.5 {
                ((2.0 * t).powi(2) * ((C2 + 1.0) * 2.0 * t - C2)) / 2.0
            } else {
                ((2.0 * t - 2.0).powi(2) * ((C2 + 1.0) * (2.0 * t - 2.0) + C2) + 2.0) / 2.0
            }
        }

        // Bounce
        Easing3Type::EaseInBounce => 1.0 - ease_out_bounce(1.0 - t),
        Easing3Type::EaseOutBounce => ease_out_bounce(t),
        Easing3Type::EaseInOutBounce => {
            if t < 0.5 {
                (1.0 - ease_out_bounce(1.0 - 2.0 * t)) / 2.0
            } else {
                (1.0 + ease_out_bounce(2.0 * t - 1.0)) / 2.0
            }
        }

        // Unknown variant — treat as linear
        _ => t,
    }
}

/// Interpolate between `start` and `end` using the given easing function.
///
/// `t` is clamped to \[0, 1\]. For `StartEndSpeed`, `a`, `b`, `c` define the
/// cubic polynomial. For all other types, they are ignored.
pub fn ease(easing_type: Easing3Type, t: f32, start: f32, end: f32, a: f32, b: f32, c: f32) -> f32 {
    let factor = evaluate_easing(easing_type, t, a, b, c);
    start + (end - start) * factor
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-5;

    fn assert_approx(actual: f32, expected: f32, msg: &str) {
        assert!(
            (actual - expected).abs() < EPSILON,
            "{msg}: expected {expected}, got {actual}"
        );
    }

    #[test]
    fn test_linear() {
        assert_approx(
            evaluate_easing(Easing3Type::Linear, 0.0, 0.0, 0.0, 0.0),
            0.0,
            "linear t=0",
        );
        assert_approx(
            evaluate_easing(Easing3Type::Linear, 0.5, 0.0, 0.0, 0.0),
            0.5,
            "linear t=0.5",
        );
        assert_approx(
            evaluate_easing(Easing3Type::Linear, 1.0, 0.0, 0.0, 0.0),
            1.0,
            "linear t=1",
        );
    }

    #[test]
    fn test_start_end_speed() {
        // a=2, b=-3, c=2 => 2t³ - 3t² + 2t
        // t=0 => 0, t=1 => 2-3+2=1, t=0.5 => 0.25-0.75+1.0=0.5
        assert_approx(
            evaluate_easing(Easing3Type::StartEndSpeed, 0.0, 2.0, -3.0, 2.0),
            0.0,
            "ses t=0",
        );
        assert_approx(
            evaluate_easing(Easing3Type::StartEndSpeed, 1.0, 2.0, -3.0, 2.0),
            1.0,
            "ses t=1",
        );
        assert_approx(
            evaluate_easing(Easing3Type::StartEndSpeed, 0.5, 2.0, -3.0, 2.0),
            0.5,
            "ses t=0.5",
        );
    }

    #[test]
    fn test_quadratic_boundaries() {
        // EaseIn: t=0 => 0, t=1 => 1
        assert_approx(
            evaluate_easing(Easing3Type::EaseInQuadratic, 0.0, 0.0, 0.0, 0.0),
            0.0,
            "quad in t=0",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseInQuadratic, 1.0, 0.0, 0.0, 0.0),
            1.0,
            "quad in t=1",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseInQuadratic, 0.5, 0.0, 0.0, 0.0),
            0.25,
            "quad in t=0.5",
        );

        // EaseOut: t=0 => 0, t=1 => 1
        assert_approx(
            evaluate_easing(Easing3Type::EaseOutQuadratic, 0.0, 0.0, 0.0, 0.0),
            0.0,
            "quad out t=0",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseOutQuadratic, 1.0, 0.0, 0.0, 0.0),
            1.0,
            "quad out t=1",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseOutQuadratic, 0.5, 0.0, 0.0, 0.0),
            0.75,
            "quad out t=0.5",
        );

        // EaseInOut: t=0 => 0, t=1 => 1
        assert_approx(
            evaluate_easing(Easing3Type::EaseInOutQuadratic, 0.0, 0.0, 0.0, 0.0),
            0.0,
            "quad inout t=0",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseInOutQuadratic, 1.0, 0.0, 0.0, 0.0),
            1.0,
            "quad inout t=1",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseInOutQuadratic, 0.5, 0.0, 0.0, 0.0),
            0.5,
            "quad inout t=0.5",
        );
    }

    #[test]
    fn test_cubic_boundaries() {
        assert_approx(
            evaluate_easing(Easing3Type::EaseInCubic, 0.0, 0.0, 0.0, 0.0),
            0.0,
            "cubic in t=0",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseInCubic, 1.0, 0.0, 0.0, 0.0),
            1.0,
            "cubic in t=1",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseOutCubic, 0.0, 0.0, 0.0, 0.0),
            0.0,
            "cubic out t=0",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseOutCubic, 1.0, 0.0, 0.0, 0.0),
            1.0,
            "cubic out t=1",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseInOutCubic, 0.0, 0.0, 0.0, 0.0),
            0.0,
            "cubic inout t=0",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseInOutCubic, 1.0, 0.0, 0.0, 0.0),
            1.0,
            "cubic inout t=1",
        );
    }

    #[test]
    fn test_quartic_boundaries() {
        assert_approx(
            evaluate_easing(Easing3Type::EaseInQuartic, 0.0, 0.0, 0.0, 0.0),
            0.0,
            "quartic in t=0",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseInQuartic, 1.0, 0.0, 0.0, 0.0),
            1.0,
            "quartic in t=1",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseOutQuartic, 0.0, 0.0, 0.0, 0.0),
            0.0,
            "quartic out t=0",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseOutQuartic, 1.0, 0.0, 0.0, 0.0),
            1.0,
            "quartic out t=1",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseInOutQuartic, 0.0, 0.0, 0.0, 0.0),
            0.0,
            "quartic inout t=0",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseInOutQuartic, 1.0, 0.0, 0.0, 0.0),
            1.0,
            "quartic inout t=1",
        );
    }

    #[test]
    fn test_quintic_boundaries() {
        assert_approx(
            evaluate_easing(Easing3Type::EaseInQuintic, 0.0, 0.0, 0.0, 0.0),
            0.0,
            "quintic in t=0",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseInQuintic, 1.0, 0.0, 0.0, 0.0),
            1.0,
            "quintic in t=1",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseOutQuintic, 0.0, 0.0, 0.0, 0.0),
            0.0,
            "quintic out t=0",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseOutQuintic, 1.0, 0.0, 0.0, 0.0),
            1.0,
            "quintic out t=1",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseInOutQuintic, 0.0, 0.0, 0.0, 0.0),
            0.0,
            "quintic inout t=0",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseInOutQuintic, 1.0, 0.0, 0.0, 0.0),
            1.0,
            "quintic inout t=1",
        );
    }

    #[test]
    fn test_back_boundaries() {
        // Back easings start at 0 and end at 1 but overshoot
        assert_approx(
            evaluate_easing(Easing3Type::EaseInBack, 0.0, 0.0, 0.0, 0.0),
            0.0,
            "back in t=0",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseInBack, 1.0, 0.0, 0.0, 0.0),
            1.0,
            "back in t=1",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseOutBack, 0.0, 0.0, 0.0, 0.0),
            0.0,
            "back out t=0",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseOutBack, 1.0, 0.0, 0.0, 0.0),
            1.0,
            "back out t=1",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseInOutBack, 0.0, 0.0, 0.0, 0.0),
            0.0,
            "back inout t=0",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseInOutBack, 1.0, 0.0, 0.0, 0.0),
            1.0,
            "back inout t=1",
        );

        // Back ease-in should go negative around t=0.3
        let mid = evaluate_easing(Easing3Type::EaseInBack, 0.3, 0.0, 0.0, 0.0);
        assert!(
            mid < 0.0,
            "back in should overshoot negative at t=0.3, got {mid}"
        );
    }

    #[test]
    fn test_bounce_boundaries() {
        assert_approx(
            evaluate_easing(Easing3Type::EaseInBounce, 0.0, 0.0, 0.0, 0.0),
            0.0,
            "bounce in t=0",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseInBounce, 1.0, 0.0, 0.0, 0.0),
            1.0,
            "bounce in t=1",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseOutBounce, 0.0, 0.0, 0.0, 0.0),
            0.0,
            "bounce out t=0",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseOutBounce, 1.0, 0.0, 0.0, 0.0),
            1.0,
            "bounce out t=1",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseInOutBounce, 0.0, 0.0, 0.0, 0.0),
            0.0,
            "bounce inout t=0",
        );
        assert_approx(
            evaluate_easing(Easing3Type::EaseInOutBounce, 1.0, 0.0, 0.0, 0.0),
            1.0,
            "bounce inout t=1",
        );
    }

    #[test]
    fn test_ease_interpolation() {
        // Linear interpolation between 10 and 20 at t=0.5 => 15
        assert_approx(
            ease(Easing3Type::Linear, 0.5, 10.0, 20.0, 0.0, 0.0, 0.0),
            15.0,
            "ease linear",
        );
        // t=0 => start
        assert_approx(
            ease(Easing3Type::Linear, 0.0, 10.0, 20.0, 0.0, 0.0, 0.0),
            10.0,
            "ease at start",
        );
        // t=1 => end
        assert_approx(
            ease(Easing3Type::Linear, 1.0, 10.0, 20.0, 0.0, 0.0, 0.0),
            20.0,
            "ease at end",
        );
    }

    #[test]
    fn test_clamping() {
        // t values outside [0,1] should be clamped
        assert_approx(
            evaluate_easing(Easing3Type::Linear, -1.0, 0.0, 0.0, 0.0),
            0.0,
            "clamp neg",
        );
        assert_approx(
            evaluate_easing(Easing3Type::Linear, 2.0, 0.0, 0.0, 0.0),
            1.0,
            "clamp pos",
        );
    }

    #[test]
    fn test_midpoint_values() {
        // EaseInQuadratic at t=0.5 => 0.25
        assert_approx(
            evaluate_easing(Easing3Type::EaseInQuadratic, 0.5, 0.0, 0.0, 0.0),
            0.25,
            "quad in mid",
        );
        // EaseOutQuadratic at t=0.5 => 0.75
        assert_approx(
            evaluate_easing(Easing3Type::EaseOutQuadratic, 0.5, 0.0, 0.0, 0.0),
            0.75,
            "quad out mid",
        );
        // EaseInCubic at t=0.5 => 0.125
        assert_approx(
            evaluate_easing(Easing3Type::EaseInCubic, 0.5, 0.0, 0.0, 0.0),
            0.125,
            "cubic in mid",
        );
        // EaseOutCubic at t=0.5 => 0.875
        assert_approx(
            evaluate_easing(Easing3Type::EaseOutCubic, 0.5, 0.0, 0.0, 0.0),
            0.875,
            "cubic out mid",
        );
        // EaseInQuartic at t=0.5 => 0.0625
        assert_approx(
            evaluate_easing(Easing3Type::EaseInQuartic, 0.5, 0.0, 0.0, 0.0),
            0.0625,
            "quartic in mid",
        );
        // EaseInQuintic at t=0.5 => 0.03125
        assert_approx(
            evaluate_easing(Easing3Type::EaseInQuintic, 0.5, 0.0, 0.0, 0.0),
            0.03125,
            "quintic in mid",
        );
    }

    #[test]
    fn test_bounce_out_midpoint() {
        // EaseOutBounce at t=0.5 (0.5 > 4/11 ≈ 0.36, < 8/11 ≈ 0.727)
        // t' = 0.5 - 6/11 ≈ -0.0454545
        // result = 7.5625 * t'^2 + 0.75
        let expected = 7.5625_f32 * (0.5_f32 - 6.0 / 11.0).powi(2) + 0.75;
        assert_approx(
            evaluate_easing(Easing3Type::EaseOutBounce, 0.5, 0.0, 0.0, 0.0),
            expected,
            "bounce out t=0.5",
        );
    }

    #[test]
    fn test_unknown_variant_treated_as_linear() {
        assert_approx(
            evaluate_easing(Easing3Type::Unknown(999), 0.5, 0.0, 0.0, 0.0),
            0.5,
            "unknown as linear",
        );
    }

    /// Test all 21 easing functions at t=0, t=0.5, and t=1 per spec section 6.6.
    #[test]
    fn test_all_21_at_boundaries_and_midpoint() {
        let types_and_expected_mid: &[(Easing3Type, f32)] = &[
            // (type, expected_at_0.5)
            (Easing3Type::StartEndSpeed, 0.5), // with a=2,b=-3,c=2
            (Easing3Type::Linear, 0.5),
            (Easing3Type::EaseInQuadratic, 0.25),
            (Easing3Type::EaseOutQuadratic, 0.75),
            (Easing3Type::EaseInOutQuadratic, 0.5),
            (Easing3Type::EaseInCubic, 0.125),
            (Easing3Type::EaseOutCubic, 0.875),
            (Easing3Type::EaseInOutCubic, 0.5),
            (Easing3Type::EaseInQuartic, 0.0625),
            (Easing3Type::EaseOutQuartic, 0.9375),
            (Easing3Type::EaseInOutQuartic, 0.5),
            (Easing3Type::EaseInQuintic, 0.03125),
            (Easing3Type::EaseOutQuintic, 0.96875),
            (Easing3Type::EaseInOutQuintic, 0.5),
        ];

        // Polynomial easings: all return 0 at t=0, 1 at t=1, and known midpoints
        for &(ty, expected_mid) in types_and_expected_mid {
            let (a, b, c) = if matches!(ty, Easing3Type::StartEndSpeed) {
                (2.0_f32, -3.0, 2.0)
            } else {
                (0.0, 0.0, 0.0)
            };
            assert_approx(
                evaluate_easing(ty, 0.0, a, b, c),
                0.0,
                &format!("{ty:?} t=0"),
            );
            assert_approx(
                evaluate_easing(ty, 1.0, a, b, c),
                1.0,
                &format!("{ty:?} t=1"),
            );
            assert_approx(
                evaluate_easing(ty, 0.5, a, b, c),
                expected_mid,
                &format!("{ty:?} t=0.5"),
            );
        }

        // Back easings: return 0 at t=0, 1 at t=1, but midpoints overshoot (no exact check)
        let back_types = [
            Easing3Type::EaseInBack,
            Easing3Type::EaseOutBack,
            Easing3Type::EaseInOutBack,
        ];
        for ty in &back_types {
            assert_approx(
                evaluate_easing(*ty, 0.0, 0.0, 0.0, 0.0),
                0.0,
                &format!("{ty:?} t=0"),
            );
            assert_approx(
                evaluate_easing(*ty, 1.0, 0.0, 0.0, 0.0),
                1.0,
                &format!("{ty:?} t=1"),
            );
            // Just confirm t=0.5 produces a finite value
            let mid = evaluate_easing(*ty, 0.5, 0.0, 0.0, 0.0);
            assert!(mid.is_finite(), "{ty:?} t=0.5 not finite: {mid}");
        }

        // Bounce easings: return 0 at t=0, 1 at t=1
        let bounce_types = [
            Easing3Type::EaseInBounce,
            Easing3Type::EaseOutBounce,
            Easing3Type::EaseInOutBounce,
        ];
        for ty in &bounce_types {
            assert_approx(
                evaluate_easing(*ty, 0.0, 0.0, 0.0, 0.0),
                0.0,
                &format!("{ty:?} t=0"),
            );
            assert_approx(
                evaluate_easing(*ty, 1.0, 0.0, 0.0, 0.0),
                1.0,
                &format!("{ty:?} t=1"),
            );
            let mid = evaluate_easing(*ty, 0.5, 0.0, 0.0, 0.0);
            assert!(mid.is_finite(), "{ty:?} t=0.5 not finite: {mid}");
        }
    }
}
