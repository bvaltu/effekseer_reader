//! F-Curve evaluation — pre-sampled animation curve interpolation.

use crate::types::enums::FCurveEdge;
use crate::types::fcurve::{
    FCurve, FCurveScalar, FCurveVector2D, FCurveVector3D, FCurveVectorColor,
};
use crate::types::primitives::{Vector2D, Vector3D};

impl FCurve {
    /// Evaluate the F-Curve at the given frame.
    ///
    /// Returns the sampled value **without** random offset applied. The caller
    /// should sample a random offset once per particle from
    /// `[offset_min, offset_max]` and add it to the returned value.
    pub fn evaluate(&self, frame: f32) -> f32 {
        let count = self.keys.len();
        if count == 0 {
            return 0.0;
        }
        if count == 1 {
            return self.keys[0];
        }

        let index = (frame - self.offset as f32) / self.freq as f32;
        let count_f = count as f32;

        // Apply start/end edge behavior
        let index = self.apply_edge(index, count_f);

        // Integer key index and fractional part
        let i = index.floor();
        let fract = index - i;
        let i = i as isize;

        if i < 0 {
            return self.keys[0];
        }
        let i = i as usize;
        if i >= count - 1 {
            return self.keys[count - 1];
        }

        // Linear interpolation between adjacent samples
        self.keys[i] + (self.keys[i + 1] - self.keys[i]) * fract
    }

    fn apply_edge(&self, index: f32, count_f: f32) -> f32 {
        if index >= 0.0 && index < count_f {
            return index;
        }

        // Determine which edge behavior to use
        let edge = if index < 0.0 {
            &self.start_edge
        } else {
            &self.end_edge
        };

        match edge {
            FCurveEdge::Constant => index.clamp(0.0, count_f - 1.0),
            FCurveEdge::Loop => {
                let mut idx = index.rem_euclid(count_f);
                // Avoid returning exactly count_f (floating point edge case)
                if idx >= count_f {
                    idx = 0.0;
                }
                idx
            }
            FCurveEdge::LoopInversely => {
                let loop_count = (index / count_f).floor() as i32;
                let mut idx = index.rem_euclid(count_f);
                if idx >= count_f {
                    idx = 0.0;
                }
                if loop_count % 2 != 0 {
                    // Reverse direction on odd loops
                    idx = (count_f - 1.0) - idx;
                }
                idx
            }
            // Unknown edge type — treat as constant
            _ => index.clamp(0.0, count_f - 1.0),
        }
    }
}

impl FCurveScalar {
    /// Evaluate the scalar F-Curve at the given frame.
    pub fn evaluate(&self, frame: f32) -> f32 {
        self.s.evaluate(frame)
    }
}

impl FCurveVector2D {
    /// Evaluate the 2D vector F-Curve at the given frame.
    pub fn evaluate(&self, frame: f32) -> Vector2D {
        Vector2D {
            x: self.x.evaluate(frame),
            y: self.y.evaluate(frame),
        }
    }
}

impl FCurveVector3D {
    /// Evaluate the 3D vector F-Curve at the given frame.
    pub fn evaluate(&self, frame: f32) -> Vector3D {
        Vector3D {
            x: self.x.evaluate(frame),
            y: self.y.evaluate(frame),
            z: self.z.evaluate(frame),
        }
    }
}

impl FCurveVectorColor {
    /// Evaluate the color F-Curve at the given frame.
    ///
    /// Returns `(r, g, b, a)` as `f32` values (typically 0–255 range, matching
    /// the original Effekseer convention).
    pub fn evaluate(&self, frame: f32) -> (f32, f32, f32, f32) {
        (
            self.r.evaluate(frame),
            self.g.evaluate(frame),
            self.b.evaluate(frame),
            self.a.evaluate(frame),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_curve(keys: Vec<f32>) -> FCurve {
        FCurve {
            start_edge: FCurveEdge::Constant,
            end_edge: FCurveEdge::Constant,
            offset_max: 0.0,
            offset_min: 0.0,
            offset: 0,
            len: keys.len() as i32,
            freq: 1,
            keys,
        }
    }

    #[test]
    fn test_empty_curve() {
        let c = make_curve(vec![]);
        assert_eq!(c.evaluate(0.0), 0.0);
    }

    #[test]
    fn test_single_key() {
        let c = make_curve(vec![5.0]);
        assert_eq!(c.evaluate(0.0), 5.0);
        assert_eq!(c.evaluate(10.0), 5.0);
    }

    #[test]
    fn test_integer_frames() {
        let c = make_curve(vec![0.0, 10.0, 20.0, 30.0]);
        assert_eq!(c.evaluate(0.0), 0.0);
        assert_eq!(c.evaluate(1.0), 10.0);
        assert_eq!(c.evaluate(2.0), 20.0);
        assert_eq!(c.evaluate(3.0), 30.0);
    }

    #[test]
    fn test_fractional_frame() {
        let c = make_curve(vec![0.0, 10.0]);
        assert!((c.evaluate(0.5) - 5.0).abs() < 1e-5);
        assert!((c.evaluate(0.25) - 2.5).abs() < 1e-5);
    }

    #[test]
    fn test_with_offset() {
        let mut c = make_curve(vec![0.0, 10.0, 20.0]);
        c.offset = 5;
        // Frame 5 maps to index 0
        assert_eq!(c.evaluate(5.0), 0.0);
        // Frame 6 maps to index 1
        assert_eq!(c.evaluate(6.0), 10.0);
        // Frame 4 is before curve — constant edge => keys[0]
        assert_eq!(c.evaluate(4.0), 0.0);
    }

    #[test]
    fn test_with_freq() {
        let mut c = make_curve(vec![0.0, 10.0, 20.0]);
        c.freq = 2;
        // Frame 0 => index 0/2 = 0
        assert_eq!(c.evaluate(0.0), 0.0);
        // Frame 2 => index 2/2 = 1
        assert_eq!(c.evaluate(2.0), 10.0);
        // Frame 1 => index 1/2 = 0.5 => lerp(0, 10, 0.5) = 5
        assert!((c.evaluate(1.0) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_constant_edge() {
        let c = make_curve(vec![10.0, 20.0, 30.0]);
        // Beyond end — should hold last value
        assert_eq!(c.evaluate(10.0), 30.0);
        // Before start — should hold first value
        assert_eq!(c.evaluate(-5.0), 10.0);
    }

    #[test]
    fn test_loop_edge() {
        let mut c = make_curve(vec![0.0, 10.0, 20.0]);
        c.end_edge = FCurveEdge::Loop;
        // Frame 3 => index 3 => loops to 0
        assert!((c.evaluate(3.0) - 0.0).abs() < 1e-5);
        // Frame 4 => index 4 => loops to 1
        assert!((c.evaluate(4.0) - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_loop_inversely_edge() {
        let mut c = make_curve(vec![0.0, 10.0, 20.0]);
        c.end_edge = FCurveEdge::LoopInversely;
        // Frame 3 => index 3, loop_count=1 (odd) => reverse: (2-0)=2 => keys[2]=20
        assert!((c.evaluate(3.0) - 20.0).abs() < 1e-5);
        // Frame 4 => index 4, loop_count=1 (odd) => reverse: (2-1)=1 => keys[1]=10
        assert!((c.evaluate(4.0) - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_negative_frame_loop() {
        let mut c = make_curve(vec![0.0, 10.0, 20.0]);
        c.start_edge = FCurveEdge::Loop;
        // Frame -3 => index -3 => rem_euclid(3) = 0
        assert!((c.evaluate(-3.0) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_with_negative_offset() {
        let mut c = make_curve(vec![0.0, 10.0, 20.0]);
        c.offset = -5;
        // Frame -5 maps to index 0
        assert_eq!(c.evaluate(-5.0), 0.0);
        // Frame -4 maps to index 1
        assert_eq!(c.evaluate(-4.0), 10.0);
        // Frame -6 is before curve — constant edge => keys[0]
        assert_eq!(c.evaluate(-6.0), 0.0);
        // Frame -3 maps to index 2
        assert_eq!(c.evaluate(-3.0), 20.0);
    }

    #[test]
    fn test_vector3d_evaluate() {
        let v = FCurveVector3D {
            timeline: 0,
            x: make_curve(vec![1.0, 2.0]),
            y: make_curve(vec![10.0, 20.0]),
            z: make_curve(vec![100.0, 200.0]),
        };
        let r = v.evaluate(0.5);
        assert!((r.x - 1.5).abs() < 1e-5);
        assert!((r.y - 15.0).abs() < 1e-5);
        assert!((r.z - 150.0).abs() < 1e-5);
    }

    #[test]
    fn test_color_evaluate() {
        let c = FCurveVectorColor {
            timeline: 0,
            r: make_curve(vec![0.0, 255.0]),
            g: make_curve(vec![255.0, 0.0]),
            b: make_curve(vec![128.0, 128.0]),
            a: make_curve(vec![255.0, 255.0]),
        };
        let (r, g, b, a) = c.evaluate(0.5);
        assert!((r - 127.5).abs() < 1e-4);
        assert!((g - 127.5).abs() < 1e-4);
        assert!((b - 128.0).abs() < 1e-4);
        assert!((a - 255.0).abs() < 1e-4);
    }

    #[test]
    fn test_scalar_evaluate() {
        let s = FCurveScalar {
            timeline: 0,
            s: make_curve(vec![0.0, 100.0]),
        };
        assert!((s.evaluate(0.5) - 50.0).abs() < 1e-5);
    }

    #[test]
    fn test_vector2d_evaluate() {
        let v = FCurveVector2D {
            timeline: 0,
            x: make_curve(vec![0.0, 10.0]),
            y: make_curve(vec![0.0, 20.0]),
        };
        let r = v.evaluate(0.5);
        assert!((r.x - 5.0).abs() < 1e-5);
        assert!((r.y - 10.0).abs() < 1e-5);
    }
}
