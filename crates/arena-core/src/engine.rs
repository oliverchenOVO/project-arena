use crate::event::{Event, EventKind, Metrics};
use crate::grid::{Grid, MapConfig};
use crate::replay::ReplayState;
use crate::{ENGINE_VERSION, SCHEMA_VERSION};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Algorithm {
    Bfs,
    Dijkstra,
    Astar,
    Dfs,
    Greedy,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Heuristic {
    Manhattan,
    Euclidean,
    Chebyshev,
    Zero,
}

impl Heuristic {
    pub fn estimate(self, grid: &Grid, node: usize) -> u64 {
        let dx = (node % grid.width).abs_diff(grid.goal % grid.width) as u64;
        let dy = (node / grid.width).abs_diff(grid.goal / grid.width) as u64;
        match self {
            Self::Manhattan => dx + dy,
            Self::Chebyshev => dx.max(dy),
            Self::Zero => 0,
            Self::Euclidean => {
                let squared = dx * dx + dy * dy;
                // Integer square root, avoiding platform-specific float rounding.
                let (mut low, mut high) = (0, dx + dy + 1);
                while low + 1 < high {
                    let mid = (low + high) / 2;
                    if mid * mid <= squared {
                        low = mid;
                    } else {
                        high = mid;
                    }
                }
                low
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunConfig {
    pub algorithm: Algorithm,
    pub algorithm_version: String,
    pub heuristic: Heuristic,
    pub neighbor_order: String,
    pub tie_break: String,
}
impl RunConfig {
    pub fn new(algorithm: Algorithm, heuristic: Heuristic) -> Self {
        Self {
            algorithm,
            algorithm_version: "1".into(),
            heuristic,
            neighbor_order: "N,E,S,W".into(),
            tie_break: "priority,h,insertion".into(),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.algorithm_version != "1"
            || self.neighbor_order != "N,E,S,W"
            || self.tie_break != "priority,h,insertion"
        {
            return Err("unsupported algorithm version or ordering policy".into());
        }
        if !matches!(self.algorithm, Algorithm::Astar | Algorithm::Greedy)
            && self.heuristic != Heuristic::Zero
        {
            return Err("BFS, DFS and Dijkstra require zero heuristic".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeMetrics {
    pub compute_ns: u64,
    pub events_generated: usize,
    pub events_per_second: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunLog {
    pub schema_version: u32,
    pub engine_version: String,
    pub run_id: String,
    pub generator_version: String,
    pub rng_version: String,
    pub terrain_costs: [u64; 4],
    pub map_config: MapConfig,
    pub grid: Grid,
    pub config: RunConfig,
    pub events: Vec<Event>,
    pub metrics: Metrics,
    pub runtime: RuntimeMetrics,
    pub canonical_hash: String,
}

fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
impl RunLog {
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&(
            self.schema_version,
            &self.engine_version,
            &self.run_id,
            &self.generator_version,
            &self.rng_version,
            &self.terrain_costs,
            &self.map_config,
            &self.grid,
            &self.config,
            &self.events,
            &self.metrics,
        ))
        .expect("serializable canonical data")
    }
    pub fn hash(&self) -> String {
        hash_bytes(&self.canonical_bytes())
    }
    /// Validate before replaying imported logs; malformed events never reach the reducer.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != SCHEMA_VERSION
            || self.engine_version != ENGINE_VERSION
            || self.generator_version != crate::stress::version(&self.map_config)
            || self.rng_version != "splitmix64/1"
            || self.terrain_costs != [1, 3, 5, 10]
        {
            return Err("unsupported log version or terrain configuration".into());
        }
        self.grid.validate()?;
        self.config.validate()?;
        if crate::stress::is_stress(self.map_config.generator) {
            crate::stress::validate(&self.map_config)?;
        }
        if self.map_config.width != self.grid.width
            || self.map_config.height != self.grid.height
            || self.map_config.wall_percent > 100
        {
            return Err("invalid map provenance".into());
        }
        if self.hash() != self.canonical_hash {
            return Err("canonical hash mismatch".into());
        }
        if !matches!(
            self.events.first().map(|e| &e.kind),
            Some(EventKind::RunStarted)
        ) || !matches!(
            self.events.last().map(|e| &e.kind),
            Some(EventKind::RunCompleted { .. })
        ) {
            return Err("log must have start/completion events".into());
        }
        let n = self.grid.cells.len();
        let mut state = ReplayState::new(n);
        let mut tick = 0;
        for (i, event) in self.events.iter().enumerate() {
            if event.sequence != i || event.tick < tick || state.completed {
                return Err("invalid event order".into());
            }
            tick = event.tick;
            let node = match &event.kind {
                EventKind::NodeDiscovered { node }
                | EventKind::NodeExpanded { node }
                | EventKind::FrontierAdded { node }
                | EventKind::FrontierRemoved { node }
                | EventKind::RelaxationCompared { node, .. }
                | EventKind::CostUpdated { node, .. }
                | EventKind::ParentUpdated { node, .. }
                | EventKind::GoalReached { node } => Some(*node),
                _ => None,
            };
            if node.is_some_and(|v| v >= n) {
                return Err("event node out of bounds".into());
            }
            match &event.kind {
                EventKind::RunStarted if i != 0 => return Err("duplicate start".into()),
                EventKind::NodeDiscovered { node } if state.nodes[*node].discovered => {
                    return Err("duplicate discovery".into())
                }
                EventKind::NodeExpanded { node }
                    if state.nodes[*node].expanded || !state.nodes[*node].discovered =>
                {
                    return Err("invalid expansion".into())
                }
                EventKind::FrontierRemoved { node } if state.nodes[*node].frontier_entries == 0 => {
                    return Err("frontier underflow".into())
                }
                EventKind::ParentUpdated { node, parent }
                    if *parent >= n || self.grid.manhattan(*node, *parent) != 1 =>
                {
                    return Err("invalid parent edge".into())
                }
                EventKind::EdgeEvaluated { from, to, cost }
                    if *from >= n
                        || *to >= n
                        || self.grid.manhattan(*from, *to) != 1
                        || self.grid.cells[*to].cost() != Some(*cost) =>
                {
                    return Err("invalid evaluated edge".into())
                }
                EventKind::PathReconstructed { path, cost } => {
                    if self.grid.path_cost(path)? != *cost {
                        return Err("invalid path cost".into());
                    }
                }
                EventKind::RunCompleted { found } if *found != !state.path.is_empty() => {
                    return Err("completion/path mismatch".into())
                }
                _ => {}
            }
            state.apply(event);
        }
        if state.accounting.metrics != self.metrics {
            return Err("metric accounting mismatch".into());
        }
        Ok(())
    }
}

pub struct Execution {
    pub log: RunLog,
    /// Independent terminal search arrays, for comparison with event-derived state.
    pub costs: Vec<Option<u64>>,
    pub parents: Vec<Option<usize>>,
    pub closed: Vec<bool>,
    pub discovered: Vec<bool>,
    pub frontier_entries: Vec<u32>,
    pub path: Vec<usize>,
}

type HeapEntry = Reverse<(u64, u64, u64, usize, u64)>;

pub fn execute(
    grid: &Grid,
    map_config: &MapConfig,
    config: &RunConfig,
) -> Result<Execution, String> {
    if crate::stress::is_stress(map_config.generator) {
        crate::stress::validate(map_config)?;
    }
    grid.validate()?;
    config.validate()?;
    if map_config.width != grid.width
        || map_config.height != grid.height
        || map_config.wall_percent > 100
    {
        return Err("invalid map provenance".into());
    }
    let run_id = hash_bytes(
        &serde_json::to_vec(&(ENGINE_VERSION, map_config, grid, config))
            .map_err(|e| e.to_string())?,
    );
    let start_time = Instant::now();
    let n = grid.cells.len();
    let mut costs = vec![None; n];
    let mut parents = vec![None; n];
    let mut closed = vec![false; n];
    let mut discovered = vec![false; n];
    let mut frontier_entries = vec![0u32; n];
    let mut events = Vec::new();
    let mut replay_state = ReplayState::new(n);
    let mut tick = 0;
    let mut emit = |kind: EventKind, tick: u64| {
        let event = Event {
            sequence: events.len(),
            tick,
            kind,
        };
        replay_state.apply(&event);
        events.push(event);
    };
    emit(EventKind::RunStarted, tick);
    let mut fifo = VecDeque::new();
    let mut heap = BinaryHeap::<HeapEntry>::new();
    let mut insertion = 0;
    discovered[grid.start] = true;
    costs[grid.start] = Some(0);
    let initial_h = config.heuristic.estimate(grid, grid.start);
    emit(EventKind::NodeDiscovered { node: grid.start }, tick);
    emit(
        EventKind::CostUpdated {
            node: grid.start,
            g: 0,
            h: initial_h,
            f: initial_h,
        },
        tick,
    );
    frontier_entries[grid.start] += 1;
    emit(EventKind::FrontierAdded { node: grid.start }, tick);
    if matches!(config.algorithm, Algorithm::Bfs | Algorithm::Dfs) {
        fifo.push_back(grid.start);
    } else {
        heap.push(Reverse((initial_h, initial_h, insertion, grid.start, 0)));
    }
    let mut path = Vec::new();
    loop {
        let entry = if matches!(config.algorithm, Algorithm::Bfs | Algorithm::Dfs) {
            (if config.algorithm == Algorithm::Dfs {
                fifo.pop_back()
            } else {
                fifo.pop_front()
            })
            .map(|node| (node, costs[node].unwrap()))
        } else {
            heap.pop().map(|Reverse((_, _, _, node, g))| (node, g))
        };
        let Some((node, queued_g)) = entry else {
            break;
        };
        frontier_entries[node] -= 1;
        emit(EventKind::FrontierRemoved { node }, tick);
        if closed[node] || costs[node] != Some(queued_g) {
            continue;
        }
        tick += 1;
        closed[node] = true;
        emit(EventKind::NodeExpanded { node }, tick);
        if node == grid.goal {
            emit(EventKind::GoalReached { node }, tick);
            let mut at = node;
            path.push(at);
            while at != grid.start {
                at = parents[at].ok_or("broken parent chain")?;
                path.push(at);
            }
            path.reverse();
            emit(
                EventKind::PathReconstructed {
                    path: path.clone(),
                    cost: queued_g,
                },
                tick,
            );
            break;
        }
        for next in grid.neighbors(node) {
            let Some(cost) = grid.cells[next].cost() else {
                continue;
            };
            let candidate = queued_g + cost;
            emit(
                EventKind::EdgeEvaluated {
                    from: node,
                    to: next,
                    cost,
                },
                tick,
            );
            emit(
                EventKind::RelaxationCompared {
                    node: next,
                    candidate,
                },
                tick,
            );
            let improve = if matches!(
                config.algorithm,
                Algorithm::Bfs | Algorithm::Dfs | Algorithm::Greedy
            ) {
                !discovered[next]
            } else {
                !closed[next] && costs[next].is_none_or(|old| candidate < old)
            };
            if !improve {
                continue;
            }
            if !discovered[next] {
                discovered[next] = true;
                emit(EventKind::NodeDiscovered { node: next }, tick);
            }
            costs[next] = Some(candidate);
            parents[next] = Some(node);
            let h = config.heuristic.estimate(grid, next);
            emit(
                EventKind::CostUpdated {
                    node: next,
                    g: candidate,
                    h,
                    f: candidate + h,
                },
                tick,
            );
            emit(
                EventKind::ParentUpdated {
                    node: next,
                    parent: node,
                },
                tick,
            );
            insertion += 1;
            frontier_entries[next] += 1;
            emit(EventKind::FrontierAdded { node: next }, tick);
            if matches!(config.algorithm, Algorithm::Bfs | Algorithm::Dfs) {
                fifo.push_back(next);
            } else {
                let priority = if config.algorithm == Algorithm::Greedy {
                    h
                } else {
                    candidate + h
                };
                heap.push(Reverse((priority, h, insertion, next, candidate)));
            }
        }
    }
    emit(
        EventKind::RunCompleted {
            found: !path.is_empty(),
        },
        tick,
    );
    let compute_ns = start_time.elapsed().as_nanos().min(u64::MAX as u128) as u64;
    let event_count = events.len();
    let mut log = RunLog {
        schema_version: SCHEMA_VERSION,
        engine_version: ENGINE_VERSION.into(),
        run_id,
        generator_version: crate::stress::version(map_config).into(),
        rng_version: "splitmix64/1".into(),
        terrain_costs: [1, 3, 5, 10],
        map_config: map_config.clone(),
        grid: grid.clone(),
        config: config.clone(),
        events,
        metrics: replay_state.accounting.metrics,
        runtime: RuntimeMetrics {
            compute_ns,
            events_generated: event_count,
            events_per_second: if compute_ns == 0 {
                0.0
            } else {
                event_count as f64 * 1e9 / compute_ns as f64
            },
        },
        canonical_hash: String::new(),
    };
    log.canonical_hash = log.hash();
    Ok(Execution {
        log,
        costs,
        parents,
        closed,
        discovered,
        frontier_entries,
        path,
    })
}
