//! 2D affine transforms in page millimeters (PDF bottom-left origin).
//!
//! Matrices use the PDF/`cm` convention: `[a b c d e f]` means
//! `x' = a*x + c*y + e`, `y' = b*x + d*y + f`.

/// Affine transform in millimeter user space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

impl Affine {
    pub const fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    pub const fn translate(tx_mm: f64, ty_mm: f64) -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: tx_mm,
            f: ty_mm,
        }
    }

    pub fn rotate_deg(degrees: f64) -> Self {
        let r = degrees.to_radians();
        let (sine, cosine) = (r.sin(), r.cos());
        Self {
            a: cosine,
            b: sine,
            c: -sine,
            d: cosine,
            e: 0.0,
            f: 0.0,
        }
    }

    pub const fn scale(sx: f64, sy: f64) -> Self {
        Self {
            a: sx,
            b: 0.0,
            c: 0.0,
            d: sy,
            e: 0.0,
            f: 0.0,
        }
    }

    pub const fn scale_uniform(s: f64) -> Self {
        Self::scale(s, s)
    }

    /// Apply `next` after `self` (point → self → next).
    pub fn then(self, next: Self) -> Self {
        // next * self
        Self {
            a: next.a * self.a + next.c * self.b,
            b: next.b * self.a + next.d * self.b,
            c: next.a * self.c + next.c * self.d,
            d: next.b * self.c + next.d * self.d,
            e: next.a * self.e + next.c * self.f + next.e,
            f: next.b * self.e + next.d * self.f + next.f,
        }
    }

    pub fn transform_point(self, x: f64, y: f64) -> (f64, f64) {
        (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        )
    }

    /// Counter-clockwise rotation of the +X axis under this linear map (degrees).
    pub fn rotation_deg(self) -> f64 {
        self.b.atan2(self.a).to_degrees()
    }

    pub fn is_finite(self) -> bool {
        [self.a, self.b, self.c, self.d, self.e, self.f]
            .into_iter()
            .all(|v| v.is_finite())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- validity ---

    #[test]
    fn translate_moves_origin() {
        let t = Affine::translate(10.0, -5.0);
        assert_eq!(t.transform_point(0.0, 0.0), (10.0, -5.0));
        assert_eq!(t.transform_point(1.0, 2.0), (11.0, -3.0));
    }

    #[test]
    fn scale_then_translate_order() {
        let m = Affine::scale_uniform(2.0).then(Affine::translate(3.0, 4.0));
        assert_eq!(m.transform_point(1.0, 1.0), (5.0, 6.0));
    }

    #[test]
    fn rotate_90_maps_unit_x_to_y() {
        let m = Affine::rotate_deg(90.0);
        let (x, y) = m.transform_point(1.0, 0.0);
        assert!(x.abs() < 1e-10);
        assert!((y - 1.0).abs() < 1e-10);
        assert!((m.rotation_deg() - 90.0).abs() < 1e-9);
    }

    // --- defect ---

    #[test]
    fn non_finite_rejected() {
        let bad = Affine {
            a: f64::NAN,
            ..Affine::identity()
        };
        assert!(!bad.is_finite());
    }
}
