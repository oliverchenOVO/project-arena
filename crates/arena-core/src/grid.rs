use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Terrain {
    Normal,
    Wall,
    Mud,
    Water,
    Heavy,
}

impl Terrain {
    pub fn cost(self) -> Option<u64> {
        match self {
            Self::Normal => Some(1),
            Self::Mud => Some(3),
            Self::Water => Some(5),
            Self::Heavy => Some(10),
            Self::Wall => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Terrain>,
    pub start: usize,
    pub goal: usize,
}

impl Grid {
    pub fn validate(&self) -> Result<(), String> {
        if self.width == 0 || self.height == 0 || self.width > 256 || self.height > 256 {
            return Err("grid dimensions must be 1..=256".into());
        }
        if self.cells.len() != self.width * self.height {
            return Err("cell count does not match dimensions".into());
        }
        if self.start >= self.cells.len() || self.goal >= self.cells.len() {
            return Err("start/goal out of bounds".into());
        }
        if self.cells[self.start] == Terrain::Wall || self.cells[self.goal] == Terrain::Wall {
            return Err("start/goal must be traversable".into());
        }
        Ok(())
    }

    pub fn neighbors(&self, node: usize) -> Vec<usize> {
        let (x, y) = (node % self.width, node / self.width);
        let mut result = Vec::with_capacity(4);
        if y > 0 {
            result.push(node - self.width);
        }
        if x + 1 < self.width {
            result.push(node + 1);
        }
        if y + 1 < self.height {
            result.push(node + self.width);
        }
        if x > 0 {
            result.push(node - 1);
        }
        result
    }

    pub fn manhattan(&self, a: usize, b: usize) -> u64 {
        ((a % self.width).abs_diff(b % self.width) + (a / self.width).abs_diff(b / self.width))
            as u64
    }

    pub fn path_cost(&self, path: &[usize]) -> Result<u64, String> {
        if path.first() != Some(&self.start) || path.last() != Some(&self.goal) {
            return Err("path endpoints invalid".into());
        }
        let mut cost = 0;
        for pair in path.windows(2) {
            if pair[1] >= self.cells.len() || self.manhattan(pair[0], pair[1]) != 1 {
                return Err("path contains invalid edge".into());
            }
            cost += self.cells[pair[1]].cost().ok_or("path crosses wall")?;
        }
        Ok(cost)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Generator {
    Open,
    Obstacles,
    Backtracker,
    Division,
    Rooms,
    Corridors,
    Weighted,
    HeuristicTrap,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapConfig {
    pub seed: u64,
    pub width: usize,
    pub height: usize,
    pub generator: Generator,
    pub wall_percent: u32,
}

impl Default for MapConfig {
    fn default() -> Self {
        Self {
            seed: 123,
            width: 32,
            height: 32,
            generator: Generator::Obstacles,
            wall_percent: 25,
        }
    }
}

/// SplitMix64, version 1. All overflow is explicitly wrapping.
pub struct SeededRng(u64);
impl SeededRng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
    pub fn below(&mut self, bound: u64) -> u64 {
        assert!(bound > 0);
        let threshold = bound.wrapping_neg() % bound;
        loop {
            let value = self.next_u64();
            if value >= threshold {
                return value % bound;
            }
        }
    }
}

pub fn generate(config: &MapConfig) -> Result<Grid, String> {
    if config.width == 0
        || config.height == 0
        || config.width > 256
        || config.height > 256
        || config.wall_percent > 100
    {
        return Err("invalid dimensions (1..256) or wall percentage (0..100)".into());
    }
    let (w, h) = (config.width, config.height);
    let mut grid = Grid {
        width: w,
        height: h,
        cells: vec![Terrain::Normal; w * h],
        start: 0,
        goal: w * h - 1,
    };
    let mut rng = SeededRng::new(config.seed);
    match config.generator {
        Generator::Open => {}
        Generator::Obstacles | Generator::Weighted => {
            for cell in &mut grid.cells {
                if rng.below(100) < config.wall_percent as u64 {
                    *cell = Terrain::Wall;
                } else if config.generator == Generator::Weighted {
                    *cell = match rng.below(10) {
                        0..=4 => Terrain::Normal,
                        5..=6 => Terrain::Mud,
                        7..=8 => Terrain::Water,
                        _ => Terrain::Heavy,
                    };
                }
            }
        }
        Generator::Backtracker => {
            grid.cells.fill(Terrain::Wall);
            grid.cells[0] = Terrain::Normal;
            let mut stack = vec![0];
            while let Some(&node) = stack.last() {
                let (x, y) = (node % w, node / w);
                let mut candidates = Vec::new();
                if y >= 2 {
                    candidates.push(node - 2 * w);
                }
                if x + 2 < w {
                    candidates.push(node + 2);
                }
                if y + 2 < h {
                    candidates.push(node + 2 * w);
                }
                if x >= 2 {
                    candidates.push(node - 2);
                }
                candidates.retain(|&n| grid.cells[n] == Terrain::Wall);
                if candidates.is_empty() {
                    stack.pop();
                } else {
                    let next = candidates[rng.below(candidates.len() as u64) as usize];
                    grid.cells[(node + next) / 2] = Terrain::Normal;
                    grid.cells[next] = Terrain::Normal;
                    stack.push(next);
                }
            }
            // Connect even-dimension border endpoints to the carved lattice.
            if w > 1 {
                grid.cells[w * h - 2] = Terrain::Normal;
            }
            if h > 1 {
                grid.cells[(h - 2) * w + w - 1] = Terrain::Normal;
            }
        }
        Generator::Division => {
            divide(&mut grid, &mut rng, 0, 0, w, h);
        }
        Generator::Rooms => {
            grid.cells.fill(Terrain::Wall);
            let count = (w * h / 50).max(2);
            let mut previous = (0, 0);
            for _ in 0..count {
                let x = rng.below(w as u64) as usize;
                let y = rng.below(h as u64) as usize;
                let rw = 3 + rng.below(6) as usize;
                let rh = 3 + rng.below(6) as usize;
                for yy in y..(y + rh).min(h) {
                    for xx in x..(x + rw).min(w) {
                        grid.cells[yy * w + xx] = Terrain::Normal;
                    }
                }
                carve_connection(&mut grid, previous, (x, y));
                previous = (x, y);
            }
            carve_connection(&mut grid, previous, (w - 1, h - 1));
        }
        Generator::Corridors => {
            grid.cells.fill(Terrain::Wall);
            for y in (0..h).step_by(2) {
                for x in 0..w {
                    grid.cells[y * w + x] = Terrain::Normal;
                }
                if y + 1 < h {
                    let x = if (y / 2) % 2 == 0 { w - 1 } else { 0 };
                    grid.cells[(y + 1) * w + x] = Terrain::Normal;
                }
            }
            if h % 2 == 0 {
                for x in 0..w {
                    grid.cells[(h - 1) * w + x] = Terrain::Normal;
                }
            }
        }
        Generator::HeuristicTrap => {
            if w >= 4 && h >= 4 {
                let x = w / 2;
                for y in 0..h - 1 {
                    grid.cells[y * w + x] = Terrain::Wall;
                }
                grid.start = w - 1;
                grid.goal = x - 1;
            }
        }
    }
    grid.cells[grid.start] = Terrain::Normal;
    grid.cells[grid.goal] = Terrain::Normal;
    grid.validate()?;
    Ok(grid)
}

fn carve_connection(grid: &mut Grid, a: (usize, usize), b: (usize, usize)) {
    for x in a.0.min(b.0)..=a.0.max(b.0) {
        grid.cells[a.1 * grid.width + x] = Terrain::Normal;
    }
    for y in a.1.min(b.1)..=a.1.max(b.1) {
        grid.cells[y * grid.width + b.0] = Terrain::Normal;
    }
}

fn divide(grid: &mut Grid, rng: &mut SeededRng, x: usize, y: usize, w: usize, h: usize) {
    if w < 3 || h < 3 {
        return;
    }
    if w > h || (w == h && rng.below(2) == 0) {
        let wx = x + 1 + 2 * rng.below(((w - 1) / 2) as u64) as usize;
        let gap = y + 2 * rng.below(h.div_ceil(2) as u64) as usize;
        for yy in y..y + h {
            if yy != gap {
                grid.cells[yy * grid.width + wx] = Terrain::Wall;
            }
        }
        divide(grid, rng, x, y, wx - x, h);
        divide(grid, rng, wx + 1, y, x + w - wx - 1, h);
    } else {
        let wy = y + 1 + 2 * rng.below(((h - 1) / 2) as u64) as usize;
        let gap = x + 2 * rng.below(w.div_ceil(2) as u64) as usize;
        for xx in x..x + w {
            if xx != gap {
                grid.cells[wy * grid.width + xx] = Terrain::Wall;
            }
        }
        divide(grid, rng, x, y, w, wy - y);
        divide(grid, rng, x, wy + 1, w, y + h - wy - 1);
    }
}
