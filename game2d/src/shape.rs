use crate::geom::{P2, V2};

pub enum RectSide {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub pos: P2,
    pub size: V2,
}

impl Rect {
    pub fn new(pos: P2, size: V2) -> Rect {
        Rect { pos, size }
    }

    #[inline]
    pub fn left(&self) -> f32 {
        self.pos.x
    }

    #[inline]
    pub fn right(&self) -> f32 {
        self.left() + self.size.x
    }

    #[inline]
    pub fn top(&self) -> f32 {
        self.pos.y
    }

    #[inline]
    pub fn bottom(&self) -> f32 {
        self.top() + self.size.y
    }

    pub fn overlaps(&self, other: &Rect) -> bool {
        !(self.right() <= other.left()
            || self.left() >= other.right()
            || self.top() >= other.bottom()
            || self.bottom() <= other.top())
    }

    pub fn touches(&self, other: &Rect) -> bool {
        !(self.right() < other.left()
            || self.left() > other.right()
            || self.top() > other.bottom()
            || self.bottom() < other.top())
    }

    pub fn collided_side(&self, rect_t0: &Rect, rect_t1: &Rect) -> RectSide {
        assert_eq!(self.overlaps(rect_t0), false);
        assert_eq!(self.overlaps(rect_t1), true);

        if rect_t0.left() >= self.right() && rect_t1.left() < self.right() {
            RectSide::Right
        } else if rect_t0.right() <= self.left() && rect_t1.right() > self.left() {
            RectSide::Left
        } else if rect_t0.top() >= self.bottom() && rect_t1.top() < self.bottom() {
            RectSide::Bottom
        } else {
            assert_eq!(
                rect_t0.bottom() <= self.top() && rect_t1.bottom() > self.top(),
                true
            );
            RectSide::Top
        }
    }
}
