use game2d::{
    self,
    collide::*,
    geom::{P2, V2},
};

use std::time::Duration;

mod test_support;
use crate::test_support::*;

#[test]
fn can_create_world_with_bodies() {
    let mut world = CollisionWorld::new();
    world.new_body(P2::new(0., 0.), V2::new(16., 16.));
    world.new_body(P2::new(0., 50.), V2::new(16., 16.));
    world.new_moving_body(P2::new(32., 32.), V2::new(16., 16.), V2::new(0., 0.));

    assert_eq!(world.bodies().count(), 3);
    assert_eq!(world.bodies_mut().count(), 3);
}

#[test]
fn can_remove_bodies() {
    let mut world = CollisionWorld::new();
    let body1 = world.new_body(P2::new(0., 0.), V2::new(16., 16.));
    let body2 = world.new_body(P2::new(0., 50.), V2::new(16., 16.));
    assert_eq!(world.bodies().count(), 2);

    world.remove_body(body1);
    assert_eq!(world.bodies().count(), 1);
    world.remove_body(body2);
    assert_eq!(world.bodies().count(), 0);
}

#[test]
fn can_query_body_with_handle() {
    let mut world = CollisionWorld::new();
    let body1 = world.new_body(P2::new(0., 0.), V2::new(16., 16.));
    let body2 = world.new_body(P2::new(0., 50.), V2::new(16., 16.));

    assert_eq!(world.body(body1).unwrap().pos, P2::new(0., 0.));
    assert_eq!(world.body_mut(body1).unwrap().pos, P2::new(0., 0.));
    assert_eq!(world.body(body2).unwrap().pos, P2::new(0., 50.));
    assert_eq!(world.body_mut(body2).unwrap().pos, P2::new(0., 50.));

    world.remove_body(body1);
    assert_eq!(world.body(body1).is_none(), true); // Query returns none if item was deleted
    assert_eq!(world.body_mut(body1).is_none(), true);
}

#[test]
fn can_mutate_bodies() {
    let mut world = CollisionWorld::new();
    let body1 = world.new_body(P2::new(0., 0.), V2::new(16., 16.));
    let body2 = world.new_body(P2::new(0., 50.), V2::new(16., 16.));

    world.body_mut(body1).unwrap().pos.x = 100.;
    world.body_mut(body2).unwrap().pos.x = 100.;

    assert_eq!(world.body(body1).unwrap().pos, P2::new(100., 0.));
    assert_eq!(world.body(body2).unwrap().pos, P2::new(100., 50.));

    for body in world.bodies_mut() {
        body.size = V2::new(10., 20.);
    }

    assert_eq!(world.body(body1).unwrap().size, V2::new(10., 20.));
    assert_eq!(world.body(body2).unwrap().size, V2::new(10., 20.));
}

/// +-------+     +-------+           +-------+-------+
/// |       |     |       |           |       |       |
/// |       | ←←← |       |  ======>  |       |       |
/// |       |     |       |           |       |       |
/// +-------+     +-------+           +-------+-------+
#[test]
fn collide_dynamic_with_single_static_body_moving_left() {
    let mut world = CollisionWorld::new();

    let wall = world.new_body(P2::new(0., 0.), V2::new(20., 20.));
    let actor = world.new_moving_body(P2::new(30., 0.), V2::new(20., 20.), V2::new(-1., 0.));

    world.elapse_time(Duration::from_secs(100));

    // Player moved until it ran into the wall and got stuck
    assert_eq_f32(world.body(actor).unwrap().pos.x, 20., 0.1);

    // The wall didn't budge
    assert_eq_f32(world.body(wall).unwrap().pos.x, 0., 0.1);

    // Bodies can be removed between world updates
    world.remove_body(wall);
    world.elapse_time(Duration::from_secs(10));
    // Player could keep moving
    assert_eq_f32(world.body(actor).unwrap().pos.x, 10., 0.1);
}

/// +-------+     +-------+           +-------+-------+
/// |       |     |       |           |       |       |
/// |       | →→→ |       |  ======>  |       |       |
/// |       |     |       |           |       |       |
/// +-------+     +-------+           +-------+-------+
#[test]
fn collide_dynamic_with_single_static_body_moving_right() {
    let mut world = CollisionWorld::new();

    let _wall = world.new_body(P2::new(30., 0.), V2::new(20., 20.));
    let actor = world.new_moving_body(P2::new(0., 0.), V2::new(20., 20.), V2::new(1., 0.));

    world.elapse_time(Duration::from_secs(100));

    // Player moved until it ran into the wall and got stuck
    assert_eq_f32(world.body(actor).unwrap().pos.x, 10., 0.1);
}

/// +-------+          +-------+
/// |       |          |       |
/// |       |          |       |
/// |       |          |       |
/// +-------+          +-------+
///     ↑              |       |
///     ↑     ======>  |       |
///     ↑              |       |
/// +-------+          +-------+
/// |       |
/// |       |
/// |       |
/// +-------+
#[test]
fn collide_dynamic_with_single_static_body_moving_up() {
    let mut world = CollisionWorld::new();

    let _wall = world.new_body(P2::new(0., 0.), V2::new(20., 20.));
    let actor = world.new_moving_body(P2::new(0., 30.), V2::new(20., 20.), V2::new(0., -1.));

    world.elapse_time(Duration::from_secs(100));

    // Player moved until it ran into the wall and got stuck
    assert_eq_f32(world.body(actor).unwrap().pos.y, 20., 0.1);
}

/// +-------+
/// |       |
/// |       |
/// |       |
/// +-------+          +-------+
///     ↓              |       |
///     ↓     ======>  |       |
///     ↓              |       |
/// +-------+          +-------+
/// |       |          |       |
/// |       |          |       |
/// |       |          |       |
/// +-------+          +-------+
#[test]
fn collide_dynamic_with_single_static_body_moving_down() {
    let mut world = CollisionWorld::new();

    let _wall = world.new_body(P2::new(0., 30.), V2::new(20., 20.));
    let actor = world.new_moving_body(P2::new(0., 0.), V2::new(20., 20.), V2::new(0., 1.));

    world.elapse_time(Duration::from_secs(100));

    // Player moved until it ran into the wall and got stuck
    assert_eq_f32(world.body(actor).unwrap().pos.y, 10., 0.1);
}

/// +-------+                         +------+
/// |       |                         |      |
/// |       |     +-------+           |      +-------+
/// |       |     |       |           |      |       |
/// +-------+ ←←← |       |  ======>  +------+       |
/// |       |     |       |           |      |       |
/// |       |     +-------+           |      +-------+
/// |       |                         |      |
/// +-------+                         +------+
///
/// Both        bodies may contribute to pushing back on the         body, so make sure we don't
/// apply too much back pressure.
#[test]
fn collide_dynamic_with_two_static_bodies() {
    let mut world = CollisionWorld::new();

    let _wall1 = world.new_body(P2::new(0., 0.), V2::new(20., 20.));
    let _wall2 = world.new_body(P2::new(0., 20.), V2::new(20., 20.));
    let actor = world.new_moving_body(P2::new(30., 10.), V2::new(20., 20.), V2::new(-1., 0.));

    world.elapse_time(Duration::from_secs(100));

    // Player moved until it ran into the walls and got stuck
    assert_eq_f32(world.body(actor).unwrap().pos.x, 20., 0.1);
}

///         +-------+                          +-------+
///         |       |                          |       |
///         |       |                          |       |
///         |       |                          |       |
/// +-------+-------+                  +-------+-------+
/// |       |                          |       |       |
/// |       |   ↖                      |       |       |
/// |       |    ↖            ======>  |       |       |
/// +-------+     ↖                    +-------+-------+
///                +-------+
///                |       |
///                |       |
///                |       |
///                +-------+
#[test]
fn collide_dynamic_into_corner() {
    let mut world = CollisionWorld::new();

    let _wall1 = world.new_body(P2::new(20., 0.), V2::new(20., 20.));
    let _wall2 = world.new_body(P2::new(0., 20.), V2::new(20., 20.));
    let actor = world.new_moving_body(P2::new(40., 40.), V2::new(20., 20.), V2::new(-1., -1.));

    world.elapse_time(Duration::from_secs(100));

    // Player moved until it ran into the walls and got stuck
    assert_eq_f32(world.body(actor).unwrap().pos.x, 20., 0.1);
    assert_eq_f32(world.body(actor).unwrap().pos.y, 20., 0.1);
}

/// +-------+                   +-------+
/// |       |                   |       |
/// |       |                   |       +-------+
/// |       | ↖                 |       |       |
/// +-------+ ↖        ======>  +-------+       |
/// |       | ↖                 |       |       |
/// |       +-------+           |       +-------+
/// |       |       |           |       |
/// +-------+       |           +-------+
///         |       |
///         +-------+
///
/// Some collision systems get stuck or hiccup on corners
#[test]
fn dynamic_body_slides_across_static_bodies() {
    let mut world = CollisionWorld::new();

    let _wall1 = world.new_body(P2::new(0., 0.), V2::new(20., 20.));
    let _wall2 = world.new_body(P2::new(0., 20.), V2::new(20., 20.));
    let actor = world.new_moving_body(P2::new(20., 30.), V2::new(20., 20.), V2::new(-1., -1.));

    world.elapse_time(Duration::from_secs(20));

    assert_eq_f32(world.body(actor).unwrap().pos.x, 20., 0.1);
    assert_eq_f32(world.body(actor).unwrap().pos.y, 10., 0.1);
}

/// +-------+     +-------+           +-------+-------+               +-------+     +-------+
/// |       |     |       |           |       |       |               |       |     |       |
/// |   A   | ←←← |   B   |  ======>  |   A   |   B   |  ======>  ←←← |   B   |     |   A   |
/// |       |     |       |           |       |       |               |       |     |       |
/// +-------+     +-------+           +-------+-------+               +-------+     +-------+
#[test]
fn can_mutate_body_to_move_it() {
    let mut world = CollisionWorld::new();

    let wall = world.new_body(P2::new(0., 0.), V2::new(20., 20.));
    let actor = world.new_moving_body(P2::new(30., 0.), V2::new(20., 20.), V2::new(-1., 0.));

    world.elapse_time(Duration::from_secs(100));

    // Player moved until it ran into the wall and got stuck
    assert_eq_f32(world.body(actor).unwrap().pos.x, 20., 0.1);

    // Bodies can be moved between world updates
    world.body_mut(wall).unwrap().pos = P2::new(100., 0.);
    world.elapse_time(Duration::from_secs(10));
    // Player could keep moving
    assert_eq_f32(world.body(actor).unwrap().pos.x, 10., 0.1);

    // Let's mutate the player to make sure they can run into the wall in its new position
    world.body_mut(actor).unwrap().vel = V2::new(1., 0.);
    world.elapse_time(Duration::from_secs(200));

    // Player moved until it ran into the wall and got stuck
    assert_eq_f32(world.body(actor).unwrap().pos.x, 80., 0.1);
}
