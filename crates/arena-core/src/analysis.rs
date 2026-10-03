use crate::grid::{Grid, Terrain};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MapFeatures {
    pub obstacle_density: f64,
    pub open_space_ratio: f64,
    pub corridor_ratio: f64,
    pub dead_end_count: usize,
    pub weighted_terrain_ratio: f64,
    pub path_tortuosity: Option<f64>,
}

pub fn features(grid: &Grid, path: &[usize]) -> MapFeatures {
    let total = grid.cells.len();
    let walls = grid.cells.iter().filter(|&&c| c == Terrain::Wall).count();
    let weighted = grid
        .cells
        .iter()
        .filter(|&&c| matches!(c, Terrain::Mud | Terrain::Water | Terrain::Heavy))
        .count();
    let (mut corridors, mut dead_ends) = (0, 0);
    for node in 0..total {
        if grid.cells[node] == Terrain::Wall {
            continue;
        }
        let degree = grid
            .neighbors(node)
            .iter()
            .filter(|&&n| grid.cells[n] != Terrain::Wall)
            .count();
        if degree == 2 {
            corridors += 1;
        }
        if degree == 1 {
            dead_ends += 1;
        }
    }
    let direct = grid.manhattan(grid.start, grid.goal);
    MapFeatures {
        obstacle_density: walls as f64 / total as f64,
        open_space_ratio: (total - walls) as f64 / total as f64,
        corridor_ratio: corridors as f64 / (total - walls).max(1) as f64,
        dead_end_count: dead_ends,
        weighted_terrain_ratio: weighted as f64 / total as f64,
        path_tortuosity: if path.is_empty() || direct == 0 {
            None
        } else {
            Some((path.len() - 1) as f64 / direct as f64)
        },
    }
}
