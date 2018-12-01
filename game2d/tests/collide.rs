use game2d::{
    self,
    collide::*,
    geom::{P2, V2},
};

use std::time::Duration;

mod test_support;
use crate::test_support::*;

const GROUP_WALL: u32 = GROUP_0;
const GROUP_ACTOR: u32 = GROUP_1;
const GROUP_PASSTHRU: u32 = GROUP_2;

fn new_default_world() -> CollisionWorld {
    CollisionWorld::new(CollisionWorldParams {
        group_pairs: vec![(GROUP_WALL, GROUP_ACTOR)],
        partition_size: (20., 20.),
    })
}

#[test]
#[should_panic(expected = "Invalid partition size: (0.0, 20.0)")]
fn partition_x_must_be_positive() {
    CollisionWorld::new(CollisionWorldParams {
        partition_size: (0., 20.),
        group_pairs: Vec::new(),
    });
}

#[test]
#[should_panic(expected = "Invalid partition size: (20.0, -20.0)")]
fn partition_y_must_be_positive() {
    CollisionWorld::new(CollisionWorldParams {
        partition_size: (20., -20.),
        group_pairs: Vec::new(),
    });
}

#[test]
fn can_create_world_with_bodies() {
    let mut world = new_default_world();
    world.new_body(GROUP_0, P2::new(0., 0.), V2::new(16., 16.));
    world.new_body(GROUP_0, P2::new(0., 50.), V2::new(16., 16.));
    world.new_moving_body(
        GROUP_0,
        P2::new(32., 32.),
        V2::new(16., 16.),
        V2::new(0., 0.),
    );

    assert_eq!(world.bodies().count(), 3);
    assert_eq!(world.bodies_mut().count(), 3);
}

#[test]
fn can_remove_bodies() {
    let mut world = new_default_world();
    let body1 = world.new_body(GROUP_0, P2::new(0., 0.), V2::new(16., 16.));
    let body2 = world.new_body(GROUP_0, P2::new(0., 50.), V2::new(16., 16.));
    assert_eq!(world.bodies().count(), 2);

    world.remove_body(body1);
    assert_eq!(world.bodies().count(), 1);
    world.remove_body(body2);
    assert_eq!(world.bodies().count(), 0);
}

#[test]
fn can_query_body_with_handle() {
    let mut world = new_default_world();
    let body1 = world.new_body(GROUP_0, P2::new(0., 0.), V2::new(16., 16.));
    let body2 = world.new_body(GROUP_0, P2::new(0., 50.), V2::new(16., 16.));

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
    let mut world = new_default_world();
    let body1 = world.new_body(GROUP_0, P2::new(0., 0.), V2::new(16., 16.));
    let body2 = world.new_body(GROUP_0, P2::new(0., 50.), V2::new(16., 16.));

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
    let mut world = new_default_world();

    let wall = world.new_body(GROUP_WALL, P2::new(0., 0.), V2::new(20., 20.));
    let actor = world.new_moving_body(
        GROUP_ACTOR,
        P2::new(30., 0.),
        V2::new(20., 20.),
        V2::new(-1., 0.),
    );

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
    let mut world = new_default_world();

    world.new_body(GROUP_WALL, P2::new(30., 0.), V2::new(20., 20.));
    let actor = world.new_moving_body(
        GROUP_ACTOR,
        P2::new(0., 0.),
        V2::new(20., 20.),
        V2::new(1., 0.),
    );

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
    let mut world = new_default_world();

    world.new_body(GROUP_WALL, P2::new(0., 0.), V2::new(20., 20.));
    let actor = world.new_moving_body(
        GROUP_ACTOR,
        P2::new(0., 30.),
        V2::new(20., 20.),
        V2::new(0., -1.),
    );

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
    let mut world = new_default_world();

    world.new_body(GROUP_WALL, P2::new(0., 30.), V2::new(20., 20.));
    let actor = world.new_moving_body(
        GROUP_ACTOR,
        P2::new(0., 0.),
        V2::new(20., 20.),
        V2::new(0., 1.),
    );

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
    let mut world = new_default_world();

    world.new_body(GROUP_WALL, P2::new(0., 0.), V2::new(20., 20.));
    world.new_body(GROUP_WALL, P2::new(0., 20.), V2::new(20., 20.));
    let actor = world.new_moving_body(
        GROUP_ACTOR,
        P2::new(30., 10.),
        V2::new(20., 20.),
        V2::new(-1., 0.),
    );

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
    let mut world = new_default_world();

    world.new_body(GROUP_WALL, P2::new(20., 0.), V2::new(20., 20.));
    world.new_body(GROUP_WALL, P2::new(0., 20.), V2::new(20., 20.));
    let actor = world.new_moving_body(
        GROUP_ACTOR,
        P2::new(40., 40.),
        V2::new(20., 20.),
        V2::new(-1., -1.),
    );

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
    let mut world = new_default_world();

    world.new_body(GROUP_WALL, P2::new(0., 0.), V2::new(20., 20.));
    world.new_body(GROUP_WALL, P2::new(0., 20.), V2::new(20., 20.));
    let actor = world.new_moving_body(
        GROUP_ACTOR,
        P2::new(20., 30.),
        V2::new(20., 20.),
        V2::new(-1., -1.),
    );

    world.elapse_time(Duration::from_secs(20));

    assert_eq_f32(world.body(actor).unwrap().pos.x, 20., 0.1);
    assert_eq_f32(world.body(actor).unwrap().pos.y, 10., 0.1);
}

/// +-------+     +-------+           +-------+-------+               +-------+     +-------+
/// |       |     |       |           |       |       |  move A       |       |     |       |
/// |   A   | ←←← |   B   |  ======>  |   A   |   B   |  ======>  ←←← |   B   |     |   A   |
/// |       |     |       |           |       |       |               |       |     |       |
/// +-------+     +-------+           +-------+-------+               +-------+     +-------+
#[test]
fn can_mutate_body_to_move_it() {
    let mut world = new_default_world();

    let wall = world.new_body(GROUP_WALL, P2::new(0., 0.), V2::new(20., 20.));
    let actor = world.new_moving_body(
        GROUP_ACTOR,
        P2::new(30., 0.),
        V2::new(20., 20.),
        V2::new(-1., 0.),
    );

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

/// +-------+     +-------+           +-----+-------+
/// |       |     |       |           |     |       |
/// |       | ←←← |       |  ======>  | ←←← |       |
/// |       |     |       |           |     |       |
/// +-------+     +-------+           +-----+-------+
#[test]
fn bodies_only_collide_if_groups_are_registered_to_collide() {
    let mut world = CollisionWorld::new(CollisionWorldParams {
        group_pairs: vec![],
        partition_size: (40., 40.),
    });

    let wall = world.new_body(GROUP_WALL, P2::new(0., 0.), V2::new(20., 20.));
    let actor = world.new_moving_body(
        GROUP_ACTOR,
        P2::new(30., 0.),
        V2::new(20., 20.),
        V2::new(-1., 0.),
    );

    world.elapse_time(Duration::from_secs(20));

    // Player passed through the wall
    assert_eq_f32(world.body(actor).unwrap().pos.x, 10., 0.1);

    // The wall didn't budge
    assert_eq_f32(world.body(wall).unwrap().pos.x, 0., 0.1);
}

#[test]
fn partitioning_the_board_optimizes_collision_performance() {
    use std::time::SystemTime;

    // Populates a world by surrounding it with walls and sprinkling walls throughout, and then
    // putting an actor at the top left moving to the bottom right.
    fn create_world(partition_size: (f32, f32)) -> CollisionWorld {
        let mut world = CollisionWorld::new(CollisionWorldParams {
            partition_size,
            group_pairs: vec![(GROUP_WALL, GROUP_ACTOR)],
        });

        let x_squares = 100;
        let y_squares = 50;
        let body_size = V2::new(20., 20.);
        let actor_vel = V2::new(10., 5.);

        for x in 0..=x_squares {
            let x = x as f32;
            world.new_body(GROUP_WALL, P2::new(x * body_size.x, 0.), body_size);
        }

        for y in 1..y_squares {
            let y = y as f32;
            world.new_body(GROUP_WALL, P2::new(0., y * body_size.y), body_size);
        }

        for y in 1..y_squares {
            let y = y as f32;
            let x_squares = x_squares as f32;
            world.new_body(
                GROUP_WALL,
                P2::new(x_squares * body_size.x, y * body_size.y),
                body_size,
            );
        }

        for x in 0..=x_squares {
            let x = x as f32;
            let y_squares = y_squares as f32;
            world.new_body(
                GROUP_WALL,
                P2::new(x * body_size.x, y_squares * body_size.y),
                body_size,
            );
        }

        for x in (10..=x_squares - 10).step_by(10) {
            for y in (5..y_squares - 5).step_by(5) {
                let x = x as f32;
                let y = y as f32;
                world.new_body(
                    GROUP_WALL,
                    P2::new(x * body_size.x, y * body_size.y),
                    body_size,
                );
            }
        }

        // Throw in some objects that should be ignored
        for x in (20..=x_squares - 20).step_by(20) {
            for y in (10..y_squares - 10).step_by(10) {
                let x = x as f32;
                let y = y as f32;
                world.new_body(
                    GROUP_PASSTHRU,
                    P2::new(x * body_size.x, y * body_size.y),
                    body_size,
                );
            }
        }

        world.new_moving_body(
            GROUP_ACTOR,
            P2::new(body_size.x, body_size.y),
            body_size,
            actor_vel,
        );

        world
    }

    fn elapse(world: &mut CollisionWorld) {
        world.elapse_time(Duration::new(20, 0));
    }

    // Small partition uses more memory but should be much faster.
    let small_partition_elapsed = {
        let mut world = create_world((40., 40.));

        let small_partition_start = SystemTime::now();
        elapse(&mut world);
        small_partition_start.elapsed().unwrap()
    };

    let large_partition_elapsed = {
        let mut world = create_world((std::f32::MAX, std::f32::MAX));

        let large_partition_start = SystemTime::now();
        elapse(&mut world);
        large_partition_start.elapsed().unwrap()
    };

    dbg!(small_partition_elapsed);
    dbg!(large_partition_elapsed);
    assert!(small_partition_elapsed < large_partition_elapsed);
}
