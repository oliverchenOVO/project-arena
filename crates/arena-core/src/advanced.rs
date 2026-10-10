//! Layer-synchronous bidirectional BFS for symmetric, unit-cost grids only.
use crate::{
    engine::{Execution, RuntimeMetrics},
    event::{Event, EventKind},
    replay::ReplayState,
    Grid, MapConfig, RunConfig, RunLog, ENGINE_VERSION, SCHEMA_VERSION,
};
use sha2::{Digest, Sha256};
use std::{collections::VecDeque, time::Instant};
pub fn bidirectional(
    grid: &Grid,
    map: &MapConfig,
    config: &RunConfig,
) -> Result<Execution, String> {
    if grid.cells.iter().any(|t| t.cost().is_some_and(|c| c != 1)) {
        return Err("Bidirectional BFS requires unweighted symmetric movement".into());
    }
    if map.width != grid.width || map.height != grid.height || map.wall_percent > 100 {
        return Err("invalid map provenance".into());
    }
    let began = Instant::now();
    let n = grid.cells.len();
    let mut events = vec![];
    let mut state = ReplayState::new(n);
    let mut emit = |kind: EventKind| {
        let e = Event {
            sequence: events.len(),
            tick: events.len() as u64,
            kind,
        };
        state.apply(&e);
        events.push(e);
    };
    emit(EventKind::RunStarted);
    let mut distances = [vec![None; n], vec![None; n]];
    let mut parents_by_side = [vec![None; n], vec![None; n]];
    let mut queues = [VecDeque::new(), VecDeque::new()];
    let mut discovered = vec![false; n];
    let mut closed = vec![false; n];
    let mut costs = vec![None; n];
    let mut parents = vec![None; n];
    let mut frontier_entries = vec![0; n];
    for (side, root) in [grid.start, grid.goal].into_iter().enumerate() {
        distances[side][root] = Some(0u64);
        queues[side].push_back(root);
        if !discovered[root] {
            discovered[root] = true;
            costs[root] = Some(0);
            emit(EventKind::NodeDiscovered { node: root });
            emit(EventKind::CostUpdated {
                node: root,
                g: 0,
                h: 0,
                f: 0,
            });
        }
        emit(EventKind::SearchSide {
            node: root,
            side: if side == 0 { "forward" } else { "backward" }.into(),
        });
        frontier_entries[root] += 1;
        emit(EventKind::FrontierAdded { node: root });
    }
    let mut meeting = if grid.start == grid.goal {
        Some(grid.start)
    } else {
        None
    };
    let mut best = u64::MAX;
    let mut side = 0;
    while meeting.is_none() && !queues[0].is_empty() && !queues[1].is_empty() {
        let layer = queues[side].len();
        for _ in 0..layer {
            let u = queues[side].pop_front().unwrap();
            frontier_entries[u] -= 1;
            emit(EventKind::FrontierRemoved { node: u });
            if !closed[u] {
                closed[u] = true;
                emit(EventKind::DirectionExpanded {
                    node: u,
                    side: if side == 0 { "forward" } else { "backward" }.into(),
                });
                emit(EventKind::NodeExpanded { node: u });
            }
            for v in grid.neighbors(u) {
                if grid.cells[v].cost().is_none() {
                    continue;
                }
                emit(EventKind::EdgeEvaluated {
                    from: u,
                    to: v,
                    cost: 1,
                });
                let candidate = distances[side][u].unwrap() + 1;
                emit(EventKind::RelaxationCompared { node: v, candidate });
                if distances[side][v].is_some() {
                    continue;
                }
                distances[side][v] = Some(candidate);
                parents_by_side[side][v] = Some(u);
                emit(EventKind::SearchSide {
                    node: v,
                    side: if side == 0 { "forward" } else { "backward" }.into(),
                });
                if !discovered[v] {
                    discovered[v] = true;
                    costs[v] = Some(candidate);
                    parents[v] = Some(u);
                    emit(EventKind::NodeDiscovered { node: v });
                    emit(EventKind::CostUpdated {
                        node: v,
                        g: candidate,
                        h: 0,
                        f: candidate,
                    });
                    emit(EventKind::ParentUpdated { node: v, parent: u });
                }
                if let Some(other) = distances[1 - side][v] {
                    let total = candidate + other;
                    if total < best {
                        best = total;
                        meeting = Some(v);
                    }
                } else {
                    queues[side].push_back(v);
                    frontier_entries[v] += 1;
                    emit(EventKind::FrontierAdded { node: v });
                }
            }
        }
        side = 1 - side;
    }
    let mut path = vec![];
    if let Some(mid) = meeting {
        emit(EventKind::MeetingPoint { node: mid });
        let mut u = mid;
        path.push(u);
        while u != grid.start {
            u = parents_by_side[0][u].ok_or("broken forward chain")?;
            path.push(u);
        }
        path.reverse();
        u = mid;
        while u != grid.goal {
            u = parents_by_side[1][u].ok_or("broken backward chain")?;
            path.push(u);
        }
        emit(EventKind::GoalReached { node: grid.goal });
        emit(EventKind::PathReconstructed {
            path: path.clone(),
            cost: grid.path_cost(&path)?,
        });
    }
    emit(EventKind::RunCompleted {
        found: !path.is_empty(),
    });
    let compute_ns = began.elapsed().as_nanos() as u64;
    let count = events.len();
    let mut log = RunLog {
        schema_version: SCHEMA_VERSION,
        engine_version: ENGINE_VERSION.into(),
        run_id: format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&(ENGINE_VERSION, map, grid, config))
                    .map_err(|e| e.to_string())?
            )
        ),
        generator_version: crate::stress::version(map).into(),
        rng_version: "splitmix64/1".into(),
        terrain_costs: [1, 3, 5, 10],
        map_config: map.clone(),
        grid: grid.clone(),
        config: config.clone(),
        events,
        metrics: state.accounting.metrics,
        runtime: RuntimeMetrics {
            compute_ns,
            events_generated: count,
            events_per_second: if compute_ns == 0 {
                0.0
            } else {
                count as f64 * 1e9 / compute_ns as f64
            },
        },
        canonical_hash: String::new(),
    };
    log.canonical_hash = log.hash();
    log.validate()?;
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
