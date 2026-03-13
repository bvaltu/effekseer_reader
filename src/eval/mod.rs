//! Runtime evaluation helpers for F-Curves, easing functions, gradients, NURBS curves,
//! and dynamic equations.
//!
//! This module is gated behind the `eval` feature flag.

pub mod dynamic_equation;
pub mod easing;
pub mod fcurve;
pub mod gradient;
pub mod nurbs;
