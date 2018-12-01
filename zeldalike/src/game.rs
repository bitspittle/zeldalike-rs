//! Main classes for processing and running the game

use ggez::conf::{Conf, WindowMode, WindowSetup};
use ggez::event::{self, EventHandler, Keycode, Mod};
use ggez::graphics::{self, Color, DrawMode, DrawParam, Image, Point2, Rect};
use ggez::timer;
use ggez::{Context, GameResult};

use game2d::collide::CollisionWorldParams;
use game2d::collide::{BodyHandle, CollisionWorld};
use game2d::geom::{P2, V2};

/// Global game settings
struct GameConfig {
    tile_size: V2,
    board_size: V2,
    win_scale: f32, // Scale board size to window size
    show_collision_outlines: bool,
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig {
            tile_size: V2::new(16., 16.),
            board_size: V2::new(160., 144.),
            win_scale: 4.,
            show_collision_outlines: false,
        }
    }
}

/// State that is modified as a result of user input
struct InputState {
    move_vec: V2,
}

impl InputState {
    fn new() -> InputState {
        InputState {
            move_vec: V2::zero(),
        }
    }

    fn handle_key(&mut self, keycode: Keycode, is_down: bool) -> bool {
        // key down: 1.0, key up: 0.0
        let is_down_val = is_down as i32 as f32; // Can't convert bool to f32 directly

        match keycode {
            Keycode::Up => {
                self.move_vec.y = -is_down_val;
            }
            Keycode::Down => {
                self.move_vec.y = is_down_val;
            }
            Keycode::Left => {
                self.move_vec.x = -is_down_val;
            }
            Keycode::Right => {
                self.move_vec.x = is_down_val;
            }
            _ => return false,
        }
        true
    }
}

/// Basic object that can be rendered to some area on the screen
struct Entity {
    pos: P2,
    size: V2,
    image: Image,
    body_handle: Option<BodyHandle>,
}

impl Entity {
    fn new(size: V2, image: Image) -> Entity {
        Entity {
            pos: P2::zero(),
            size,
            image,
            body_handle: None,
        }
    }

    fn center_on_board(&mut self, board_size: V2) {
        self.pos = ((board_size - self.size) / 2.).into();
    }

    fn set_tile_pos(&mut self, tile_size: V2, tile_index_x: i32, tile_index_y: i32) {
        let tile_pos = tile_size * (tile_index_x as f32, tile_index_y as f32);
        self.pos = tile_pos.into();
    }

    fn draw(&self, win_scale: f32, ctx: &mut Context) -> GameResult<()> {
        let scaled_pos = self.pos * win_scale;

        // Scale image so it fits (e.g. a 64x64 image on a 32x32 entity -> 0.5x0.5 scale)
        let image_size = (self.image.width() as f32, self.image.height() as f32);
        let image_ratio = (self.size * win_scale) / image_size;

        let draw_params = DrawParam {
            dest: Point2::new(scaled_pos.x, scaled_pos.y),
            scale: Point2::new(image_ratio.x, image_ratio.y),
            ..Default::default()
        };

        graphics::draw_ex(ctx, &self.image, draw_params)
    }
}

const GROUP_WALL: u32 = game2d::collide::GROUP_0;
const GROUP_PLYR: u32 = game2d::collide::GROUP_1;

/// Collection of ALL state related to rendering the game - essentially,
/// represents the game world.
struct GameState {
    config: GameConfig,
    input: InputState,
    collision_world: CollisionWorld,
    player: Entity,
    walls: Vec<Entity>,
}

impl GameState {
    #[allow(clippy::new_ret_no_self)] // Returns Result<Self> instead of Self
    fn new(cfg: GameConfig, ctx: &mut Context) -> GameResult<GameState> {
        let player_image = Image::new(ctx, "/images/player.png")?;
        let wall_image = Image::new(ctx, "/images/wall.png")?;
        let mut collision_world = CollisionWorld::new(CollisionWorldParams {
            group_pairs: vec![(GROUP_WALL, GROUP_PLYR)],
            partition_size: (20., 20.),
        });

        let mut player = Entity::new(cfg.tile_size, player_image);
        player.center_on_board(cfg.board_size);
        player.body_handle = Some(collision_world.new_body(GROUP_PLYR, player.pos, player.size));

        let mut walls: Vec<Entity> = Vec::new();

        let num_tiles_x = (cfg.board_size.x / cfg.tile_size.x) as i32;
        let num_tiles_y = (cfg.board_size.y / cfg.tile_size.y) as i32;

        for tile_x in 0..num_tiles_x {
            let mut wall = Entity::new(cfg.tile_size, wall_image.clone());
            wall.set_tile_pos(cfg.tile_size, tile_x as i32, 0);
            walls.push(wall)
        }

        for tile_y in 1..(num_tiles_y - 1) {
            {
                let mut wall = Entity::new(cfg.tile_size, wall_image.clone());
                wall.set_tile_pos(cfg.tile_size, 0, tile_y as i32);
                walls.push(wall)
            }
            {
                let mut wall = Entity::new(cfg.tile_size, wall_image.clone());
                wall.set_tile_pos(cfg.tile_size, (num_tiles_x - 1) as i32, tile_y as i32);
                walls.push(wall)
            }
        }

        for tile_x in 0..num_tiles_x {
            let mut wall = Entity::new(cfg.tile_size, wall_image.clone());
            wall.set_tile_pos(cfg.tile_size, tile_x as i32, (num_tiles_y - 1) as i32);
            walls.push(wall)
        }

        for wall in &walls {
            collision_world.new_body(GROUP_WALL, wall.pos, wall.size);
        }

        Ok(GameState {
            config: cfg,
            input: InputState::new(),
            collision_world,
            player,
            walls,
        })
    }

    fn render_collision_outlines(&mut self, ctx: &mut Context) {
        for body in self.collision_world.bodies() {
            let pos_scaled = body.pos * self.config.win_scale;
            let size_scaled = body.size * self.config.win_scale;

            let _ = graphics::rectangle(
                ctx,
                DrawMode::Line(1.),
                Rect::new(pos_scaled.x, pos_scaled.y, size_scaled.x, size_scaled.y),
            );
        }

        let player_handle = self.player.body_handle.unwrap();
        let mut touching = self.collision_world.get_touching(player_handle);
        if !touching.is_empty() {
            touching.push(self.collision_world.body(player_handle).unwrap());
            let restore_color = graphics::get_color(ctx);
            let _ = graphics::set_color(ctx, Color::from_rgb(255, 0, 0));

            for body in touching {
                let pos_scaled = body.pos * self.config.win_scale;
                let size_scaled = body.size * self.config.win_scale;

                let _ = graphics::rectangle(
                    ctx,
                    DrawMode::Line(2.),
                    Rect::new(pos_scaled.x, pos_scaled.y, size_scaled.x, size_scaled.y),
                );
            }
            let _ = graphics::set_color(ctx, restore_color);
        }
    }
}

impl EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let player_handle = self.player.body_handle.unwrap();
        {
            let body = self.collision_world.body_mut(player_handle).unwrap();
            body.vel = self.input.move_vec.normalized() * (70.);
        }

        self.collision_world.elapse_time(timer::get_delta(ctx));

        {
            let body = self.collision_world.body(player_handle).unwrap();
            self.player.pos = body.pos;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        let win_scale = self.config.win_scale;

        for wall in &self.walls {
            wall.draw(win_scale, ctx)?;
        }
        self.player.draw(win_scale, ctx)?;
        if self.config.show_collision_outlines {
            self.render_collision_outlines(ctx);
        }

        graphics::present(ctx);
        timer::yield_now();
        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: Keycode, _keymod: Mod, _repeat: bool) {
        if !self.input.handle_key(keycode, true) {
            match keycode {
                Keycode::Escape => ctx.quit().unwrap(),
                Keycode::Tab => {
                    self.config.show_collision_outlines = !self.config.show_collision_outlines
                }
                _ => {}
            }
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: Keycode, _keymod: Mod, _repeat: bool) {
        self.input.handle_key(keycode, false);
    }
}

/// Home for the `run` method that sets up the game and starts its main loop.
pub struct Game;

impl Game {
    pub fn run() {
        let cfg = GameConfig::default();

        let win_size = cfg.board_size * cfg.win_scale;
        let c = Conf {
            window_setup: WindowSetup {
                title: "Zeldalike.rs: A Gamedev Tutorial for Rust".to_owned(),
                ..Default::default()
            },
            window_mode: WindowMode::default().dimensions(win_size.x as u32, win_size.y as u32),
            ..Default::default()
        };
        let ctx = &mut Context::load_from_conf("zeldalike", "bitspittle", c).unwrap();

        // Background color taken from Godot
        graphics::set_background_color(ctx, Color::from_rgb(77, 77, 77));

        let state = &mut GameState::new(cfg, ctx).unwrap();
        event::run(ctx, state).unwrap();
    }
}
