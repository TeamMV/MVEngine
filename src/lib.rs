#![feature(portable_simd)]
#![feature(ptr_metadata)]
#![feature(try_trait_v2)]
#![deny(clippy::unwrap_used, clippy::redundant_closure_for_method_calls)]
#![cfg_attr(not(debug_assertions), deny(clippy::todo))]
#![warn(clippy::pedantic)]


// pub mod net;
pub mod ui;
pub mod window;
pub mod math;
pub mod rendering;
pub mod color;
pub mod input;
pub mod ecs;
