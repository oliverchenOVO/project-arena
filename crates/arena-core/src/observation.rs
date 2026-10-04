//! Facts, arithmetic and rule-based interpretation are intentionally separate.
use crate::{event::EventKind, replay::ReplayState, Algorithm, RunLog};
use serde_json::{json, Value};
pub fn thresholds() -> Value {
    serde_json::from_str(include_str!("../../../config/stress-thresholds.json"))
        .expect("versioned thresholds")
}
pub fn ratio(a: u64, b: u64) -> Option<f64> {
    if b == 0 {
        None
    } else {
        Some(a as f64 / b as f64)
    }
}
pub fn markers(log: &RunLog) -> Vec<Value> {
    let mut result = Vec::new();
    let (mut frontier, mut peak, mut peak_cursor, mut updates) = (0usize, 0usize, 0usize, 0u64);
    let milestones = [
        log.metrics.cost_updates / 4,
        log.metrics.cost_updates / 2,
        log.metrics.cost_updates * 3 / 4,
    ];
    for e in &log.events {
        let label = match &e.kind {
            EventKind::NodeDiscovered { node } if *node == log.grid.goal => Some("Goal discovered"),
            EventKind::NodeExpanded { node } if *node == log.grid.goal => Some("Goal expanded"),
            EventKind::PathReconstructed { .. } => Some("Path reconstructed"),
            EventKind::CostUpdated { .. } => {
                updates += 1;
                if updates > 0 && milestones.contains(&updates) {
                    Some("Cost update milestone")
                } else {
                    None
                }
            }
            EventKind::FrontierAdded { .. } => {
                frontier += 1;
                if frontier > peak {
                    peak = frontier;
                    peak_cursor = e.sequence + 1;
                }
                None
            }
            EventKind::FrontierRemoved { .. } => {
                frontier = frontier.saturating_sub(1);
                None
            }
            _ => None,
        };
        if let Some(label) = label {
            result.push(json!({"label":label,"cursor":e.sequence+1,"event":e.sequence}));
        }
    }
    if peak > 0 {
        result.push(
            json!({"label":"Max frontier","cursor":peak_cursor,"event":peak_cursor-1,"value":peak}),
        );
    }
    result.sort_by_key(|v| v["cursor"].as_u64().unwrap());
    result
}
pub fn history(log: &RunLog, node: usize, cursor: usize) -> Value {
    let mut events = Vec::new();
    let mut changes = 0;
    let mut parent = None;
    for e in log.events.iter().take(cursor) {
        let relevant = match e.kind {
            EventKind::NodeDiscovered { node: n }
            | EventKind::NodeExpanded { node: n }
            | EventKind::FrontierAdded { node: n }
            | EventKind::FrontierRemoved { node: n }
            | EventKind::CostUpdated { node: n, .. } => n == node,
            EventKind::ParentUpdated { node: n, parent: p } if n == node => {
                if parent.is_some() && parent != Some(p) {
                    changes += 1;
                }
                parent = Some(p);
                true
            }
            _ => false,
        };
        if relevant {
            events.push(e);
        }
    }
    json!({"events":events,"parent_changes":changes})
}
pub fn diff(left: &ReplayState, right: &ReplayState) -> Value {
    let (mut shared, mut left_only, mut right_only, mut visited) = (0, 0, 0, 0);
    for (a, b) in left.nodes.iter().zip(&right.nodes) {
        match (a.expanded, b.expanded) {
            (true, true) => shared += 1,
            (true, false) => left_only += 1,
            (false, true) => right_only += 1,
            _ => {}
        }
        if a.discovered && b.discovered {
            visited += 1;
        }
    }
    let overlap = left.path.iter().filter(|n| right.path.contains(n)).count();
    json!({"shared_expanded":shared,"left_only":left_only,"right_only":right_only,"both_visited":visited,"path_overlap":overlap,"left_path_only":left.path.len()-overlap,"right_path_only":right.path.len()-overlap})
}
pub fn observations(
    logs: &[RunLog],
    baseline: Option<&RunLog>,
    step_optimal: Option<u64>,
) -> Value {
    let optimal_cost = baseline
        .or_else(|| logs.iter().find(|l| l.config.algorithm == Algorithm::Astar))
        .and_then(|l| l.metrics.path_cost);
    let t = thresholds();
    let best = logs.iter().map(|l| l.metrics.expanded).min().unwrap_or(0);
    let mut measured = Vec::new();
    let mut derived = Vec::new();
    let mut interpretation = Vec::new();
    for log in logs {
        let m = &log.metrics;
        let r = baseline.and_then(|b| ratio(m.expanded, b.metrics.expanded));
        let mut sum = 0u64;
        for e in &log.events {
            if let EventKind::NodeExpanded { node } = e.kind {
                sum += log.config.heuristic.estimate(&log.grid, node);
            }
        }
        let cost_ratio = m.path_cost.zip(optimal_cost).and_then(|(a, b)| {
            if b > 0 {
                Some(a as f64 / b as f64)
            } else {
                None
            }
        });
        let depth = m.path_length;
        let branch = depth
            .filter(|d| *d > 0)
            .map(|d| (m.expanded as f64).powf(1.0 / d as f64));
        let (key, effect, threshold_key) = match log.config.algorithm {
            Algorithm::Dfs => ("dfs", ratio(m.expanded, best), "ratio"),
            Algorithm::Greedy => ("greedy", cost_ratio, "cost_ratio"),
            Algorithm::Astar => ("astar", r, "expansion_ratio"),
            Algorithm::Bfs => ("bfs", cost_ratio, "cost_ratio"),
            Algorithm::Dijkstra => ("dijkstra", None, "ratio"),
        };
        let badge = effect.map(|v| {
            if v >= t[key][format!("strong_{threshold_key}")]
                .as_f64()
                .unwrap_or(f64::INFINITY)
            {
                "STRONGLY EXPOSED"
            } else if v
                >= t[key][format!("moderate_{threshold_key}")]
                    .as_f64()
                    .unwrap_or(f64::INFINITY)
            {
                "MODERATELY EXPOSED"
            } else {
                "WEAK EFFECT"
            }
        });
        let optimal = m.path_cost.zip(optimal_cost).map(|(a, b)| a == b);
        measured.push(json!({"algorithm":log.config.algorithm,"heuristic":log.config.heuristic,"metrics":m,"initial_h":log.config.heuristic.estimate(&log.grid,log.grid.start),"expanded_h_sum":sum}));
        derived.push(json!({"algorithm":log.config.algorithm,"heuristic":log.config.heuristic,"average_h_at_expansion":if m.expanded>0{Some(sum as f64/m.expanded as f64)}else{None},"effective_branching_proxy":branch,"expansion_ratio_to_dijkstra":r,"heuristic_guidance_ratio":r.map(|v|1.0-v),"path_cost_optimal":optimal,"step_optimal":m.path_length.zip(step_optimal).map(|(a,b)|a==b),"cost_ratio_to_optimal":cost_ratio,"expanded_vs_best":ratio(m.expanded,best),"stress_effect":effect,"badge":badge}));
        let text = match log.config.algorithm {
            Algorithm::Bfs => {
                "BFS optimizes unweighted step count; weighted path cost is a different objective."
            }
            Algorithm::Astar if r.is_some_and(|v| v >= 0.9) => {
                "The heuristic gives little expansion reduction on this layout."
            }
            Algorithm::Astar if optimal == Some(true) && r.is_some_and(|v| v < 1.0) => {
                "A* achieved the same optimal cost with fewer expansions than Dijkstra."
            }
            Algorithm::Greedy if cost_ratio.is_some_and(|v| v > 1.0) => {
                "Heuristic-only ordering produced a higher-cost route on this layout."
            }
            Algorithm::Dfs if effect.is_some_and(|v| v > 1.25) => {
                "Depth-first ordering explored more cells than the best measured contender."
            }
            _ => "The measured run does not establish a strong weakness for this comparison.",
        };
        interpretation.push(
            json!({"algorithm":log.config.algorithm,"rule_version":"observation/1","text":text}),
        );
    }
    json!({"version":"observation/1","problem":logs.first().map(|l|&l.map_config),"measured":measured,"derived":derived,"interpretation":interpretation,"baseline":{"algorithm":baseline.map(|b|b.config.algorithm),"optimal_cost_reference":baseline.or_else(||logs.iter().find(|l|l.config.algorithm==Algorithm::Astar)).map(|l|l.config.algorithm),"expanded":baseline.map(|b|b.metrics.expanded),"optimal_cost":optimal_cost,"step_optimal_length":step_optimal,"hash":baseline.map(|b|&b.canonical_hash)},"thresholds":t,"formulas":{"guidance":"1 - expanded / Dijkstra expanded","branching_proxy":"expanded^(1 / solution edges); proxy, not solved effective branching factor","cost_overhead":"cost / optimal cost - 1","admissibility":"All four integer heuristics are admissible and consistent for four-way movement with destination costs >= 1; this does not make Greedy optimal."}})
}
