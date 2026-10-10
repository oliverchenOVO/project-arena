//! UI-independent deterministic execution, generation and replay.
pub mod advanced;
pub mod analysis;
pub mod bridge;
pub mod domain;
pub mod domain_replay;
pub mod engine;
pub mod event;
pub mod graph;
pub mod grid;
pub mod observation;
pub mod observe;
pub mod replay;
pub mod sorting;
pub mod stress;
pub mod validation;

pub use engine::{execute, Algorithm, Heuristic, RunConfig, RunLog};
pub use grid::{generate, Generator, Grid, MapConfig, Terrain};

pub const ENGINE_VERSION: &str = "arena-engine/1";
pub const SCHEMA_VERSION: u32 = 1;
