//! This relative simple collision module owns `CollisionWorld` and related structs. With these, you
//! can create bodies and allow the system to manage their interactions.

use std::time::Duration;

use crate::grid::GridCoord;
use crate::{
    geom::{P2, V2},
    grid::Grid,
    grid::GridRegion,
    pool::{Handle as PoolHandle, Pool},
    shape::{Rect, RectSide},
};
use std::collections::HashMap;
use std::collections::HashSet;

const fn group(i: u32) -> u32 {
    1 << i
}
pub const GROUP_0: u32 = group(0);
pub const GROUP_1: u32 = group(1);
pub const GROUP_2: u32 = group(2);
pub const GROUP_3: u32 = group(3);
pub const GROUP_4: u32 = group(4);
pub const GROUP_5: u32 = group(5);
pub const GROUP_6: u32 = group(6);
pub const GROUP_7: u32 = group(7);
pub const GROUP_8: u32 = group(8);
pub const GROUP_9: u32 = group(9);
pub const GROUP_10: u32 = group(10);
pub const GROUP_11: u32 = group(11);
pub const GROUP_12: u32 = group(12);
pub const GROUP_13: u32 = group(13);
pub const GROUP_14: u32 = group(14);
pub const GROUP_15: u32 = group(15);
pub const GROUP_16: u32 = group(16);
pub const GROUP_17: u32 = group(17);
pub const GROUP_18: u32 = group(18);
pub const GROUP_19: u32 = group(19);
pub const GROUP_20: u32 = group(20);
pub const GROUP_21: u32 = group(21);
pub const GROUP_22: u32 = group(22);
pub const GROUP_23: u32 = group(23);
pub const GROUP_25: u32 = group(25);
pub const GROUP_26: u32 = group(26);
pub const GROUP_27: u32 = group(27);
pub const GROUP_28: u32 = group(28);
pub const GROUP_29: u32 = group(29);
pub const GROUP_30: u32 = group(30);
pub const GROUP_31: u32 = group(31);

/// An object in space which can interact with other objects. A `Body` should act as the source of
/// truth for a game object's position in the world, as it will respect the space taken up by other
/// bodies.
#[derive(Debug)]
pub struct Body {
    pub group: u32,
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
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct BodyHandle {
    inner_handle: PoolHandle, // Our own handle just delegates all work
}

/// An owner of several bodies. After creating one and adding several bodies to it, use
/// `elapse_time` to update the world's state frame by frame.
pub struct CollisionWorld {
    time_counter: Duration,
    time_step: Duration,
    bodies: Pool<Body>,
    /// A mapping of the source group to all groups they can collide with
    group_masks: HashMap<u32, u32>,
    /// How large we want our grid partitions to be. This is an optimization as it allows us to only
    /// check our own (and nearby) partitions for object we might collide with, potentially ignoring
    /// many others. Larger partitions use less memory but smaller partitions should provide a
    /// significant performance boost.
    ///
    /// A reasonable value for this is one that divides the screen up into 12 or so areas, but
    /// profiling / experimentation may be worthwhile here.
    partition_size: (f32, f32),
    grid: Grid<PoolHandle>,
    /// Users may modify bodies externally by accessing them through mutating getters. In those
    /// cases, we attempt to refresh them next time we get a chance.
    refresh_handles: HashSet<PoolHandle>,
    /// We keep track of moving bodies, since they are the only ones that can initiate a collision;
    /// in our update loop, we only have to process what they are doing.
    moving_handles: HashSet<PoolHandle>,
}

impl<'b> From<&'b Body> for Rect {
    fn from(b: &'b Body) -> Self {
        Rect::new(b.pos, b.size)
    }
}

pub struct CollisionWorldParams {
    /// A list of group pairs that can collide with each other. This relationship is automatically
    /// symmetric: If `A` can collide with `B` then `B` will also collide with `A`
    pub group_pairs: Vec<(u32, u32)>,
    /// See comment for `CollisionWorld.partition_size`
    pub partition_size: (f32, f32),
}
#[allow(clippy::new_without_default)] // API is intentionally explicit
impl CollisionWorld {
    pub fn new(params: CollisionWorldParams) -> CollisionWorld {
        if params.partition_size.0 <= 0. || params.partition_size.1 <= 0. {
            panic!("Invalid partition size: {:?}", params.partition_size)
        }

        let mut group_masks = HashMap::new();
        for group_pair in params.group_pairs.iter() {
            let group_a = group_pair.0;
            let group_b = group_pair.1;
            *group_masks.entry(group_a).or_insert(group_b) |= group_b;
            *group_masks.entry(group_b).or_insert(group_a) |= group_a;
        }

        CollisionWorld {
            time_counter: Duration::from_millis(0),
            time_step: Duration::from_micros(16666), // 16.67 ms, roughly 60 fps
            group_masks,
            bodies: Pool::new(),
            partition_size: params.partition_size,
            grid: Grid::new(),
            refresh_handles: HashSet::new(),
            moving_handles: HashSet::new(),
        }
    }

    pub fn new_body(&mut self, group: u32, pos: P2, size: V2) -> BodyHandle {
        self.new_moving_body(group, pos, size, V2::zero())
    }

    /// Convenience method for calling `new_body` with non-zero velocity
    pub fn new_moving_body(&mut self, group: u32, pos: P2, size: V2, vel: V2) -> BodyHandle {
        let body = Body {
            group,
            pos,
            size,
            vel,
        };

        let handle = BodyHandle {
            inner_handle: self.bodies.push(body),
        };
        self.grid
            .insert(handle.inner_handle, self.create_region(pos, size));

        if !vel.is_zero() {
            self.moving_handles.insert(handle.inner_handle);
        }

        handle
    }

    pub fn remove_body(&mut self, handle: BodyHandle) {
        self.bodies.remove(handle.inner_handle);
        self.grid.remove(handle.inner_handle);
    }

    pub fn body(&self, handle: BodyHandle) -> Option<&Body> {
        self.bodies.get(handle.inner_handle)
    }

    pub fn body_mut(&mut self, handle: BodyHandle) -> Option<&mut Body> {
        self.refresh_handles.insert(handle.inner_handle);
        self.moving_handles.remove(&handle.inner_handle);
        self.bodies.get_mut(handle.inner_handle)
    }

    pub fn bodies(&self) -> impl Iterator<Item = &Body> {
        self.bodies.iter()
    }

    pub fn bodies_mut(&mut self) -> impl Iterator<Item = &mut Body> {
        self.bodies.handles().for_each(|h| {
            self.refresh_handles.insert(h);
        });
        self.moving_handles.clear();
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

    fn get_region_bodies(&self, region: GridRegion, exclude: PoolHandle) -> Vec<&Body> {
        self.grid
            .query(region)
            .iter()
            .filter(|&&h| *h != exclude)
            .filter_map(|&h| self.bodies.get(*h))
            .collect()
    }

    pub fn elapse_time(&mut self, duration: Duration) {
        if !self.refresh_handles.is_empty() {
            for refresh_handle in self.refresh_handles.iter() {
                if let Some(body) = self.bodies.get(*refresh_handle) {
                    self.grid
                        .insert(*refresh_handle, self.create_region(body.pos, body.size));

                    if !body.vel.is_zero() {
                        self.moving_handles.insert(*refresh_handle);
                    }
                }
            }
            self.refresh_handles.clear();
        }

        self.time_counter += duration;
        let time_step_secs = self.time_step.subsec_micros() as f32 / 1_000_000f32;
        while self.time_counter >= self.time_step {
            self.time_counter -= self.time_step;

            {
                for moving_handle in self.moving_handles.iter() {
                    let moving_body = self.bodies.get(*moving_handle).unwrap();
                    let group_masks = *self.group_masks.get(&moving_body.group).unwrap_or(&0);
                    let rect_t0 = Rect::from(moving_body);
                    let vel_step = moving_body.vel * time_step_secs;
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

                    let dynamic_region = GridRegion::bounding(
                        self.create_region(rect_t0.pos, rect_t0.size),
                        self.create_region(rect_t0.pos + vel_step, rect_t1.size),
                    );
                    let nearby_bodies = self.get_region_bodies(dynamic_region, *moving_handle);

                    // If we are trying to move horizontally, check if we collide first
                    if vel_step.x != 0. {
                        rect_t1.pos.x = rect_t0.pos.x + vel_step.x;
                        for nearby_body in &nearby_bodies {
                            if nearby_body.group & group_masks == 0 {
                                continue;
                            }
                            let rect_curr = Rect::from(*nearby_body);

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
                        for nearby_body in &nearby_bodies {
                            if nearby_body.group & group_masks == 0 {
                                continue;
                            }

                            let rect_curr = Rect::from(*nearby_body);

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

                    self.grid.insert(
                        *moving_handle,
                        self.create_region(rect_t1.pos, rect_t1.size),
                    );
                    self.bodies.get_mut(*moving_handle).unwrap().pos = rect_t1.pos;
                }
            }
        }
    }

    fn create_region(&self, pos: P2, size: V2) -> GridRegion {
        let tl = pos / self.partition_size;
        let br = (pos + size) / self.partition_size;
        let coord_tl: GridCoord = (tl.x as i16, tl.y as i16).into();
        let coord_br: GridCoord = (br.x as i16, br.y as i16).into();

        (coord_tl, coord_br).into()
    }
}
