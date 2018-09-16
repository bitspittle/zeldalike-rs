//! Main classes for processing and running the game

use game2d::geom::{P2, V2};

use ggez::conf::{Conf, WindowMode, WindowSetup};
use ggez::event::{self, EventHandler, Keycode, Mod};
use ggez::graphics::{self, Color, DrawParam, Image, Point2};
use ggez::{Context, GameResult};

/// Global game settings
struct GameConfig {
    tile_size: V2,
    board_size: V2,
    win_scale: f32, // Scale board size to window size
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig {
            tile_size: V2::new(16., 16.),
            board_size: V2::new(160., 144.),
            win_scale: 4.,
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
}

impl Entity {
    fn new(size: V2, image: Image) -> Entity {
        Entity {
            pos: P2::zero(),
            size,
            image,
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

/// Collection of ALL state related to rendering the game - essentially,
/// represents the game world.
struct GameState {
    config: GameConfig,
    input: InputState,
    player: Entity,
    walls: Vec<Entity>,
}

impl GameState {
    #[allow(clippy::new_ret_no_self)] // Returns Result<Self> instead of Self
    fn new(cfg: GameConfig, ctx: &mut Context) -> GameResult<GameState> {
        let player_image = Image::new(ctx, "/images/player.png")?;
        let wall_image = Image::new(ctx, "/images/wall.png")?;

        let mut player = Entity::new(cfg.tile_size, player_image);
        player.center_on_board(cfg.board_size);

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

        Ok(GameState {
            config: cfg,
            input: InputState::new(),
            player,
            walls,
        })
    }
}

impl EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        if self.input.move_vec != V2::zero() {
            self.player.pos += self.input.move_vec.normalized();
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

        graphics::present(ctx);
        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: Keycode, _keymod: Mod, _repeat: bool) {
        if !self.input.handle_key(keycode, true) {
            if let Keycode::Escape = keycode {
                ctx.quit().unwrap()
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
