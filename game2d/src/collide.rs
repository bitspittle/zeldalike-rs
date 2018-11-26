//! This relative simple collision module owns `CollisionWorld` and related structs. With these, you
//! can create bodies and allow the system to manage their interactions.

use std::time::Duration;

use crate::geom::{P2, V2};
use crate::pool::{Handle as PoolHandle, Pool};
use crate::shape::{Rect, RectSide};

/// An object in space which can interact with other objects. A `Body` should act as the source of
/// truth for a game object's position in the world, as it will respect the space taken up by other
/// bodies.
#[derive(Debug)]
pub struct Body {
    pub pos: P2,
    pub size: V2,
    pub vel: V2,
}

impl PartialEq for Body {
    fn eq(&self, other: &Body) -> bool {
        // Bodies are unique - they'll only equal each other if they are the exact same in memory
        self as *const _ == other as *const _
    }
}
impl Eq for Body {}

/// A handle to a `Body`, which `CollisionWorld` creates for you when you ask it to create a body. You
/// use the handle to safely query / remove bodies.
#[derive(Debug, Copy, Clone)]
pub struct BodyHandle {
    inner_handle: PoolHandle, // Our own handle just delegates all work
}

/// An owner of several bodies. After creating one and adding several bodies to it, use
/// `elapse_time` to update the world's state frame by frame.
pub struct CollisionWorld {
    time_counter: Duration,
    time_step: Duration,
    bodies: Pool<Body>,
}

impl<'b> From<&'b Body> for Rect {
    fn from(b: &'b Body) -> Self {
        Rect::new(b.pos, b.size)
    }
}

#[allow(clippy::new_without_default)] // API is intentionally explicit
impl CollisionWorld {
    pub fn new() -> CollisionWorld {
        CollisionWorld {
            time_counter: Duration::from_millis(0),
            time_step: Duration::from_micros(16666), // 16.67 ms, roughly 60 fps
            bodies: Pool::new(),
        }
    }

    pub fn new_body(&mut self, pos: P2, size: V2) -> BodyHandle {
        self.new_moving_body(pos, size, V2::zero())
    }

    /// Convenience method for calling `new_body` with non-zero velocity
    pub fn new_moving_body(&mut self, pos: P2, size: V2, vel: V2) -> BodyHandle {
        let body = Body { pos, size, vel };

        BodyHandle {
            inner_handle: self.bodies.push(body),
        }
    }

    pub fn remove_body(&mut self, handle: BodyHandle) {
        self.bodies.remove(handle.inner_handle);
    }

    pub fn body(&self, handle: BodyHandle) -> Option<&Body> {
        self.bodies.get(handle.inner_handle)
    }

    pub fn body_mut(&mut self, handle: BodyHandle) -> Option<&mut Body> {
        self.bodies.get_mut(handle.inner_handle)
    }

    pub fn bodies(&self) -> impl Iterator<Item = &Body> {
        self.bodies.iter()
    }

    pub fn bodies_mut(&mut self) -> impl Iterator<Item = &mut Body> {
        self.bodies.iter_mut()
    }

    pub fn get_touching(&self, handle: BodyHandle) -> Vec<&Body> {
        let mut touching = Vec::with_capacity(0); // Don't allocate by default

        if let Some(body) = self.body(handle) {
            let rect_body = Rect::from(body);
            for other_body in self.bodies() {
                if !other_body.vel.is_zero() || body == other_body {
                    continue; // Only collide moving with non-moving bodies (for now...)
                }

                let rect_other = Rect::from(other_body);
                if rect_body.touches(&rect_other) {
                    touching.push(other_body);
                }
            }
        }

        touching
    }

    pub fn elapse_time(&mut self, duration: Duration) {
        self.time_counter += duration;
        let time_step_secs = self.time_step.subsec_micros() as f32 / 1_000_000f32;
        while self.time_counter >= self.time_step {
            self.time_counter -= self.time_step;

            {
                // Break bodies up into non-moving and moving instances
                let (static_handles, dynamic_handles): (Vec<PoolHandle>, Vec<PoolHandle>) = self
                    .bodies
                    .handles()
                    .partition(|&h| self.bodies.get(h).unwrap().vel.is_zero());

                for dynamic_handle in &dynamic_handles {
                    let dynamic_body = self.bodies.get(*dynamic_handle).unwrap();
                    let rect_t0 = Rect::from(dynamic_body);
                    let vel_step = dynamic_body.vel * time_step_secs;
                    let mut rect_t1 = rect_t0;

                    // In more complex collision systems, you need to handle arbitrary shapes
                    // bumping against other arbitrary shapes at any angle. In our case, however,
                    // we only have rectangles moving on a grid. This lets us simplify our collision
                    // logic immensely (as long as our requirements don't change in the future...)
                    //
                    // In order to avoid bodies getting stuck on corners (and other tricky things
                    // which can happen when a body is moving at an angle), we break down each
                    // body's movement into x- and y- components, and resolve collisions in two
                    // passes.

                    // TODO: Optimize collision checking by adding collision groups, space
                    //  partitioning, and keeping better track of moving bodies.

                    // If we are trying to move horizontally, check if we collide first
                    if vel_step.x != 0. {
                        rect_t1.pos.x = rect_t0.pos.x + vel_step.x;
                        for static_handle in &static_handles {
                            let static_body = self.bodies.get(*static_handle).unwrap();
                            let rect_curr = Rect::from(static_body);

                            if rect_curr.overlaps(&rect_t1) {
                                match rect_curr.collided_side(&rect_t0, &rect_t1) {
                                    RectSide::Left => {
                                        rect_t1.pos.x = rect_curr.left() - rect_t1.size.x
                                    }
                                    RectSide::Right => rect_t1.pos.x = rect_curr.right(),
                                    _ => {}
                                }
                            }
                        }
                    }

                    // If we are trying to move vertically, check if we collide first
                    if vel_step.y != 0. {
                        rect_t1.pos.y = rect_t0.pos.y + vel_step.y;
                        for static_handle in &static_handles {
                            let static_body = self.bodies.get(*static_handle).unwrap();
                            let rect_curr = Rect::from(static_body);

                            if rect_curr.overlaps(&rect_t1) {
                                match rect_curr.collided_side(&rect_t0, &rect_t1) {
                                    RectSide::Top => {
                                        rect_t1.pos.y = rect_curr.top() - rect_t1.size.y
                                    }
                                    RectSide::Bottom => rect_t1.pos.y = rect_curr.bottom(),
                                    _ => {}
                                }
                            }
                        }
                    }

                    self.bodies.get_mut(*dynamic_handle).unwrap().pos = rect_t1.pos
                }
            }
        }
    }
}
