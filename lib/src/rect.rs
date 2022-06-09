//! Behold, the power of the rectangle!

use glam::f32::{Affine2, Vec2};

use serde::{Deserialize, Serialize};

/// A rectangle represented by two opposite corners of a box.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Rect {
    pub p1: Vec2,
    pub p2: Vec2,
}

impl Rect {
    /// Creates a new `Rect` from two points.
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Rect {
        Rect {
            p1: Vec2::new(x1, y1),
            p2: Vec2::new(x2, y2),
        }
    }

    /// Creates a new `Rect` from a bottom-left point and a width height.
    pub fn new_wh(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect {
            p1: Vec2::new(x, y),
            p2: Vec2::new(x, y) + Vec2::new(w, h),
        }
    }

    /// The rightmost bound of the box.
    pub fn right(&self) -> f32 {
        self.p1.x.max(self.p2.x)
    }

    /// The leftmost bound of the box.
    pub fn left(&self) -> f32 {
        self.p1.x.min(self.p2.x)
    }

    /// The topmost bound of the box.
    ///
    /// Top is the *larger* float. **NOT** GL space.
    pub fn top(&self) -> f32 {
        self.p1.y.max(self.p2.y)
    }

    /// The bottommost bound of the box.
    ///
    /// Bottom is the *smaller* float. **NOT** GL space.
    pub fn bottom(&self) -> f32 {
        self.p1.y.min(self.p2.y)
    }

    /// The width of the box.
    pub fn width(&self) -> f32 {
        self.right() - self.left()
    }

    /// The height of the box.
    pub fn height(&self) -> f32 {
        self.top() - self.bottom()
    }

    /// The center of the box.
    pub fn center(&self) -> Vec2 {
        (self.p1 + self.p2) / 2.
    }

    /// Test collision between two `Rect`s using AABB.
    pub fn collides(&self, other: &Rect) -> bool {
        self.left() < other.right()
            && self.right() > other.left()
            && self.bottom() < other.top()
            && self.top() > other.bottom()
    }

    /// Translates the `Rect` by a given vector.
    pub fn translate(self, translation: Vec2) -> Rect {
        self.transform(Affine2::from_translation(translation))
    }

    /// Scales the `Rect` by a given vector about the origin.
    pub fn scale(self, scale: Vec2) -> Rect {
        let center = self.center();
        self.scale_about(scale, center)
    }

    /// Scales the `Rect` by a given vector about a given point.
    pub fn scale_about(self, scale: Vec2, origin: Vec2) -> Rect {
        self.transform(Affine2::from_translation(-origin))
            .transform(Affine2::from_scale(scale))
            .transform(Affine2::from_translation(origin))
    }

    /// Transforms the `Rect` by a given transformation matrix.
    ///
    /// More specifically, it transforms the two points that define the
    /// rectangle. This function does not work properly for rotation and
    /// shearing. You are encouraged to use the more specific
    /// [`Rect::translate`] and [`Rect::scale`] functions.
    pub fn transform(mut self, transform: Affine2) -> Rect {
        self.p1 = transform.transform_point2(self.p1);
        self.p2 = transform.transform_point2(self.p2);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aabb() {
        let rect1 = Rect::new_wh(Vec2::ZERO, Vec2::ONE);
        let rect2 = Rect::new_wh(Vec2::new(0., 1.), Vec2::ONE);
        let rect3 = Rect::new_wh(Vec2::new(0.5, 0.5), Vec2::ONE);
        let rect4 = Rect::new_wh(Vec2::new(2., -2.), Vec2::ONE);

        assert!(!rect1.collides(&rect2));
        assert!(rect1.collides(&rect3));
        assert!(!rect1.collides(&rect4));

        assert!(rect2.collides(&rect3));
        assert!(!rect2.collides(&rect4));

        assert!(!rect3.collides(&rect4));

        assert!(rect1.collides(&rect1));
    }
}
