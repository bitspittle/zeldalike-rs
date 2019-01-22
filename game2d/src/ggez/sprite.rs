use crate::geom::{P2, V2};
use ggez::{
    graphics::{self, DrawParam, Image, Point2, Rect},
    Context, GameResult,
};
use std::rc::Rc;

pub struct SpriteSheet {
    pub image: Image,
    pub tile_size: V2,
    pub num_tiles: (u16, u16),
}
pub struct SpriteParams {
    pub sheet: Rc<SpriteSheet>,
    pub curr_tile: Option<(u16, u16)>,
    pub pos: Option<P2>,
}
impl SpriteParams {
    pub fn new(sheet: &Rc<SpriteSheet>) -> SpriteParams {
        SpriteParams {
            sheet: sheet.clone(),
            curr_tile: None,
            pos: None,
        }
    }
    pub fn curr_tile(mut self, curr_tile: (u16, u16)) -> SpriteParams {
        self.curr_tile = Some(curr_tile);
        self
    }

    pub fn pos(mut self, pos: P2) -> SpriteParams {
        self.pos = Some(pos);
        self
    }
}
pub struct Sprite {
    pub sheet: Rc<SpriteSheet>,
    pub curr_tile: (u16, u16),
    pub pos: P2,
}

impl Sprite {
    pub fn new(params: SpriteParams) -> Sprite {
        Sprite {
            sheet: params.sheet,
            curr_tile: params.curr_tile.unwrap_or((0, 0)),
            pos: params.pos.unwrap_or_default(),
        }
    }

    pub fn size(&self) -> V2 {
        self.sheet.tile_size
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let tile_wh = (
            self.sheet.tile_size.x / self.sheet.image.width() as f32,
            self.sheet.tile_size.y / self.sheet.image.height() as f32,
        );
        let tile_xy = (
            self.curr_tile.0 as f32 * tile_wh.0,
            self.curr_tile.1 as f32 * tile_wh.1,
        );

        let draw_params = DrawParam {
            src: Rect::new(tile_xy.0, tile_xy.1, tile_wh.0, tile_wh.1),
            dest: Point2::new(self.pos.x, self.pos.y),
            ..Default::default()
        };

        graphics::draw_ex(ctx, &self.sheet.image, draw_params)
    }
}
