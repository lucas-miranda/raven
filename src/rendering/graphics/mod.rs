mod triangle;

mod grid;
pub use grid::Grid;

use super::{
    DrawConfig,
    RenderState,
};

pub trait Graphic {
    fn draw<'d>(
        &'d self,
        state: &'d mut dyn RenderState,
        draw_config: DrawConfig,
    );
}
