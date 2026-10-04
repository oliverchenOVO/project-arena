//! UI-independent deterministic execution, generation and replay.
pub mod analysis;
pub mod bridge;
pub mod engine;
pub mod event;
pub mod grid;
pub mod observation;
pub mod replay;
pub mod stress;
pub mod validation;

pub use engine::{execute, Algorithm, Heuristic, RunConfig, RunLog};
pub use grid::{generate, Generator, Grid, MapConfig, Terrain};

pub const ENGINE_VERSION: &str = "arena-engine/1";
pub const SCHEMA_VERSION: u32 = 1;
