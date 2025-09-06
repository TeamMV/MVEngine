pub mod ecs;
pub mod events;
pub mod language;
pub mod timing;
pub mod physics;
pub mod fs;

pub trait WorstCase {
    fn unrecoverable(&self) -> !;
}