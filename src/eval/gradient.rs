//! Gradient sampling — interpolate color and alpha keys.

use crate::types::gradient::Gradient;

impl Gradient {
    /// Sample the gradient at position `t` (0.0 to 1.0).
    ///
    /// Returns `(r, g, b, a)` where RGB has intensity pre-multiplied.
    /// `t` is clamped to \[0, 1\].
    pub fn sample(&self, t: f32) -> (f32, f32, f32, f32) {
        let t = t.clamp(0.0, 1.0);
        let (r, g, b, intensity) = self.sample_color(t);
        let alpha = self.sample_alpha(t);
        (r * intensity, g * intensity, b * intensity, alpha)
    }

    fn sample_color(&self, t: f32) -> (f32, f32, f32, f32) {
        if self.colors.is_empty() {
            return (0.0, 0.0, 0.0, 1.0);
        }
        if self.colors.len() == 1 || t <= self.colors[0].position {
            let k = &self.colors[0];
            return (k.r, k.g, k.b, k.intensity);
        }
        let last = &self.colors[self.colors.len() - 1];
        if t >= last.position {
            return (last.r, last.g, last.b, last.intensity);
        }

        // Find bracketing keys
        for i in 0..self.colors.len() - 1 {
            let a = &self.colors[i];
            let b = &self.colors[i + 1];
            if t >= a.position && t <= b.position {
                let range = b.position - a.position;
                if range <= 0.0 {
                    return (a.r, a.g, a.b, a.intensity);
                }
                let fract = (t - a.position) / range;
                return (
                    a.r + (b.r - a.r) * fract,
                    a.g + (b.g - a.g) * fract,
                    a.b + (b.b - a.b) * fract,
                    a.intensity + (b.intensity - a.intensity) * fract,
                );
            }
        }

        // Fallback: return last key
        (last.r, last.g, last.b, last.intensity)
    }

    fn sample_alpha(&self, t: f32) -> f32 {
        if self.alphas.is_empty() {
            return 1.0;
        }
        if self.alphas.len() == 1 || t <= self.alphas[0].position {
            return self.alphas[0].alpha;
        }
        let last = &self.alphas[self.alphas.len() - 1];
        if t >= last.position {
            return last.alpha;
        }

        for i in 0..self.alphas.len() - 1 {
            let a = &self.alphas[i];
            let b = &self.alphas[i + 1];
            if t >= a.position && t <= b.position {
                let range = b.position - a.position;
                if range <= 0.0 {
                    return a.alpha;
                }
                let fract = (t - a.position) / range;
                return a.alpha + (b.alpha - a.alpha) * fract;
            }
        }

        last.alpha
    }
}

#[cfg(test)]
mod tests {
    use crate::types::gradient::{Gradient, GradientAlphaKey, GradientColorKey};

    const EPSILON: f32 = 1e-5;

    fn assert_approx(actual: f32, expected: f32, msg: &str) {
        assert!(
            (actual - expected).abs() < EPSILON,
            "{msg}: expected {expected}, got {actual}"
        );
    }

    fn make_red_blue_gradient() -> Gradient {
        Gradient {
            colors: vec![
                GradientColorKey {
                    position: 0.0,
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    intensity: 1.0,
                },
                GradientColorKey {
                    position: 1.0,
                    r: 0.0,
                    g: 0.0,
                    b: 1.0,
                    intensity: 1.0,
                },
            ],
            alphas: vec![
                GradientAlphaKey {
                    position: 0.0,
                    alpha: 1.0,
                },
                GradientAlphaKey {
                    position: 1.0,
                    alpha: 1.0,
                },
            ],
        }
    }

    #[test]
    fn test_red_blue_midpoint() {
        let g = make_red_blue_gradient();
        let (r, _g, b, a) = g.sample(0.5);
        assert_approx(r, 0.5, "red at midpoint");
        assert_approx(b, 0.5, "blue at midpoint");
        assert_approx(a, 1.0, "alpha at midpoint");
    }

    #[test]
    fn test_endpoints() {
        let g = make_red_blue_gradient();
        let (r, _, b, _) = g.sample(0.0);
        assert_approx(r, 1.0, "red at start");
        assert_approx(b, 0.0, "blue at start");

        let (r, _, b, _) = g.sample(1.0);
        assert_approx(r, 0.0, "red at end");
        assert_approx(b, 1.0, "blue at end");
    }

    #[test]
    fn test_clamping() {
        let g = make_red_blue_gradient();
        let (r, _, b, _) = g.sample(-1.0);
        assert_approx(r, 1.0, "clamped below");
        assert_approx(b, 0.0, "clamped below");

        let (r, _, b, _) = g.sample(2.0);
        assert_approx(r, 0.0, "clamped above");
        assert_approx(b, 1.0, "clamped above");
    }

    #[test]
    fn test_intensity() {
        let g = Gradient {
            colors: vec![
                GradientColorKey {
                    position: 0.0,
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    intensity: 2.0,
                },
                GradientColorKey {
                    position: 1.0,
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    intensity: 2.0,
                },
            ],
            alphas: vec![GradientAlphaKey {
                position: 0.0,
                alpha: 1.0,
            }],
        };
        let (r, g_val, b, _) = g.sample(0.5);
        assert_approx(r, 2.0, "intensity r");
        assert_approx(g_val, 2.0, "intensity g");
        assert_approx(b, 2.0, "intensity b");
    }

    #[test]
    fn test_alpha_interpolation() {
        let g = Gradient {
            colors: vec![GradientColorKey {
                position: 0.0,
                r: 1.0,
                g: 1.0,
                b: 1.0,
                intensity: 1.0,
            }],
            alphas: vec![
                GradientAlphaKey {
                    position: 0.0,
                    alpha: 0.0,
                },
                GradientAlphaKey {
                    position: 1.0,
                    alpha: 1.0,
                },
            ],
        };
        let (_, _, _, a) = g.sample(0.5);
        assert_approx(a, 0.5, "alpha midpoint");
    }

    #[test]
    fn test_empty_gradient() {
        let g = Gradient {
            colors: vec![],
            alphas: vec![],
        };
        let (r, g_val, b, a) = g.sample(0.5);
        // Empty colors => (0,0,0) with intensity 1.0 => (0*1, 0*1, 0*1)
        assert_approx(r, 0.0, "empty r");
        assert_approx(g_val, 0.0, "empty g");
        assert_approx(b, 0.0, "empty b");
        assert_approx(a, 1.0, "empty alpha default");
    }

    #[test]
    fn test_three_key_gradient() {
        let g = Gradient {
            colors: vec![
                GradientColorKey {
                    position: 0.0,
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    intensity: 1.0,
                },
                GradientColorKey {
                    position: 0.5,
                    r: 0.0,
                    g: 1.0,
                    b: 0.0,
                    intensity: 1.0,
                },
                GradientColorKey {
                    position: 1.0,
                    r: 0.0,
                    g: 0.0,
                    b: 1.0,
                    intensity: 1.0,
                },
            ],
            alphas: vec![GradientAlphaKey {
                position: 0.0,
                alpha: 1.0,
            }],
        };
        // At 0.25 — between red and green
        let (r, g_val, b, _) = g.sample(0.25);
        assert_approx(r, 0.5, "three-key r at 0.25");
        assert_approx(g_val, 0.5, "three-key g at 0.25");
        assert_approx(b, 0.0, "three-key b at 0.25");
    }
}
