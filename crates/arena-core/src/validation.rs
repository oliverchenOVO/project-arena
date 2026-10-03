//! Acceptance checks against search-owned arrays and independent prefix replay.
use crate::engine::Execution;
use crate::grid::SeededRng;
use crate::replay::{Replay, ReplayState};

pub fn verify_execution(execution: &Execution, seek_samples: usize) -> Result<usize, String> {
    let log = &execution.log;
    log.validate()?;
    let mut replay = Replay::new(&log.events, log.grid.cells.len());
    replay.seek(log.events.len())?;
    let final_state = &replay.state;
    for (i, node) in final_state.nodes.iter().enumerate() {
        if node.g != execution.costs[i]
            || node.parent != execution.parents[i]
            || node.expanded != execution.closed[i]
            || node.discovered != execution.discovered[i]
            || node.frontier_entries != execution.frontier_entries[i]
        {
            return Err(format!("execution/replay node mismatch at {i}"));
        }
    }
    if final_state.path != execution.path || final_state.accounting.metrics != log.metrics {
        return Err("terminal replay mismatch".into());
    }
    let mut targets = vec![0, log.events.len()];
    for checkpoint in
        (replay.checkpoint_interval..log.events.len()).step_by(replay.checkpoint_interval)
    {
        targets.extend([checkpoint - 1, checkpoint, checkpoint + 1]);
    }
    let mut rng = SeededRng::new(log.map_config.seed ^ 0xabc123);
    for _ in 0..seek_samples {
        targets.push(rng.below((log.events.len() + 1) as u64) as usize);
    }
    let case_count = targets.len();
    // A single sorted prefix pass builds expectations without quadratic work.
    targets.sort_unstable();
    let mut prefix = ReplayState::new(log.grid.cells.len());
    let mut cursor = 0;
    for target in targets {
        for event in &log.events[cursor..target] {
            prefix.apply(event);
        }
        cursor = target;
        replay.seek(target)?;
        if replay.state != prefix {
            return Err(format!("seek divergence at {target}"));
        }
    }
    // Exercise backward seeks and event stepping, too.
    replay.seek(log.events.len())?;
    if replay.step_backward() {
        replay.step_forward();
        if replay.state != prefix {
            return Err("backward/forward divergence".into());
        }
    }
    Ok(case_count)
}

/// Independent O(V²) shortest-cost oracle, deliberately not the execution heap.
pub fn reference_cost(grid: &crate::Grid, uniform_steps: bool) -> Option<u64> {
    let n = grid.cells.len();
    let mut distances = vec![u64::MAX; n];
    let mut visited = vec![false; n];
    distances[grid.start] = 0;
    for _ in 0..n {
        let node = (0..n)
            .filter(|&i| !visited[i] && distances[i] != u64::MAX)
            .min_by_key(|&i| (distances[i], i))?;
        if node == grid.goal {
            return Some(distances[node]);
        }
        visited[node] = true;
        // Deliberately enumerate all candidates instead of sharing neighbors().
        for next in 0..n {
            if grid.manhattan(node, next) != 1 {
                continue;
            }
            if let Some(cost) = grid.cells[next].cost() {
                distances[next] =
                    distances[next].min(distances[node] + if uniform_steps { 1 } else { cost });
            }
        }
    }
    None
}
