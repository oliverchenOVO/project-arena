//! Versioned deterministic scenarios; tendencies, never a worst-case claim.
use crate::grid::{Generator, Grid, MapConfig, SeededRng, Terrain};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StressParams {
    pub version: String,
    pub branch_count: usize,
    pub branch_depth: usize,
    pub goal_placement: usize,
    pub corridor_width: usize,
    pub orientation: usize,
}
impl Default for StressParams {
    fn default() -> Self {
        Self {
            version: "stress/1".into(),
            branch_count: 8,
            branch_depth: 24,
            goal_placement: 28,
            corridor_width: 1,
            orientation: 0,
        }
    }
}
pub fn is_stress(g: Generator) -> bool {
    matches!(
        g,
        Generator::DfsDeepTrap
            | Generator::GreedyTrap
            | Generator::LowInformation
            | Generator::BfsWeightedLesson
    )
}
pub fn version(map: &MapConfig) -> &str {
    if is_stress(map.generator) {
        "stress/1"
    } else {
        "grid/1"
    }
}
pub fn validate(map: &MapConfig) -> Result<(), String> {
    let p = map.stress.clone().unwrap_or_default();
    let (w, h) = (map.width, map.height);
    if !(8..=128).contains(&w)
        || !(8..=128).contains(&h)
        || map.wall_percent > 100
        || p.version != "stress/1"
        || !(1..=64).contains(&p.branch_count)
        || !(1..=126).contains(&p.branch_depth)
        || !(1..=4).contains(&p.corridor_width)
        || p.orientation > 3
        || p.goal_placement < 2
        || p.goal_placement > 126
    {
        return Err("stress/1 requires dimensions 8..128, branches 1..64, depth 1..126, goal 2..126, corridor 1..4, orientation 0..3".into());
    }
    if p.orientation % 2 == 1 && w != h {
        return Err("odd rotations require a square grid".into());
    }
    Ok(())
}
pub fn generate(map: &MapConfig) -> Result<Grid, String> {
    validate(map)?;
    let p = map.stress.clone().unwrap_or_default();
    let (w, h) = (map.width, map.height);
    let mut rng = SeededRng::new(map.seed);
    let mut g = Grid {
        width: w,
        height: h,
        cells: vec![Terrain::Wall; w * h],
        start: w + 1,
        goal: w + w - 2,
    };
    let mut carve = |x: usize, y: usize| {
        g.cells[y * w + x] = Terrain::Normal;
    };
    match map.generator {
        Generator::DfsDeepTrap => {
            let end = p.goal_placement.min(w - 2);
            for x in 1..=end {
                for y in 1..=p.corridor_width.min(h - 2) {
                    carve(x, y);
                }
            }
            let spacing = p.corridor_width + 1;
            for x in (1..end).step_by(spacing).take(p.branch_count) {
                let depth = p
                    .branch_depth
                    .saturating_sub(rng.below(4) as usize)
                    .max(1)
                    .min(h - 3);
                for y in 2..=depth + 1 {
                    for xx in x..(x + p.corridor_width).min(end) {
                        carve(xx, y);
                    }
                }
            }
            g.goal = w + end;
        }
        Generator::GreedyTrap => {
            // Two openings in a long barrier. A horizontal lure extends toward
            // the goal but only connects through the distant bottom opening.
            let mid = w / 2;
            let sy = 2 + rng.below((h / 4).max(1) as u64) as usize;
            for y in 0..h {
                for x in 0..w {
                    carve(x, y);
                }
            }
            for y in 1..h - 1 {
                g.cells[y * w + mid] = Terrain::Wall;
            }
            // The top entrance requires moving west first around a side wall.
            let side = mid.saturating_sub(3).max(1);
            for x in side..mid {
                g.cells[(sy - 1) * w + x] = Terrain::Wall;
            }
            // The visually tempting upper opening also has a terrain toll.
            // Greedy ignores g; optimal searches can choose the longer cheap opening.
            for y in 0..sy {
                for x in side..=mid + 1 {
                    if g.cells[y * w + x] != Terrain::Wall {
                        g.cells[y * w + x] = Terrain::Heavy;
                    }
                }
            }
            g.start = sy * w + mid - 1;
            g.goal = sy * w + mid + 1;
        }
        Generator::LowInformation => {
            // Seeded serpentine corridor: geometric distance cannot describe detours.
            for y in (1..h - 1).step_by(2) {
                for x in 1..w - 1 {
                    carve(x, y);
                }
                if y + 2 < h - 1 {
                    let x = if ((y - 1) / 2) % 2 == 0 { w - 2 } else { 1 };
                    carve(x, y + 1);
                }
            }
            let last = if (h - 2) % 2 == 1 { h - 2 } else { h - 3 };
            g.goal = last * w + if ((last - 1) / 2) % 2 == 0 { w - 2 } else { 1 };
            // Short dead ends vary the measured expansion ratio across seeds.
            for y in (2..last).step_by(2) {
                let x = 2 + rng.below((w - 4) as u64) as usize;
                g.cells[y * w + x] = Terrain::Normal;
            }
        }
        Generator::BfsWeightedLesson => {
            let y = h / 2;
            for x in 1..w - 1 {
                carve(x, y);
                carve(x, y + 2);
            }
            carve(1, y + 1);
            carve(w - 2, y + 1);
            g.start = y * w + 1;
            g.goal = y * w + w - 2;
            for x in 2..w - 2 {
                g.cells[y * w + x] = if rng.below(2) == 0 {
                    Terrain::Heavy
                } else {
                    Terrain::Water
                };
            }
        }
        _ => return Err("not a stress generator".into()),
    }
    // Rectangular-safe quarter turns preserve terrain and endpoints exactly.
    for _ in 0..p.orientation {
        let transform = |i: usize| (i % g.width) * g.height + (g.height - 1 - i / g.width);
        let mut cells = vec![Terrain::Wall; g.cells.len()];
        for (i, c) in g.cells.iter().enumerate() {
            cells[transform(i)] = *c;
        }
        g = Grid {
            width: g.height,
            height: g.width,
            start: transform(g.start),
            goal: transform(g.goal),
            cells,
        };
    }
    // Keep provenance dimensions invariant when rotating rectangular inputs.
    if g.width != w || g.height != h {
        return Err("odd rotations require a square grid".into());
    }
    g.cells[g.start] = Terrain::Normal;
    g.cells[g.goal] = Terrain::Normal;
    g.validate()?;
    Ok(g)
}
