mod continuous;
pub use continuous::ContinuousLoop;

use crate::{window::WindowContext, ecs::Domain};

pub trait ApplicationLoop {
    fn new(window_context: WindowContext) -> Self;
    fn register_domain<D: Into<Box<dyn Domain>>>(&mut self, domain: D);
    fn run(self);
}
