use arena_core::replay::{Replay, ReplayState};
use arena_core::validation::verify_execution;
use arena_core::{execute, generate, Algorithm, Generator, Heuristic, MapConfig, RunConfig};
use serde_json::{json, Value};
use std::time::Instant;
pub fn gate() -> Result<Value, String> {
    let mut rows = Vec::new();
    let mut hash_checks = 0;
    let mut seeks = 0;
    for family in [
        Generator::DfsDeepTrap,
        Generator::GreedyTrap,
        Generator::LowInformation,
        Generator::BfsWeightedLesson,
    ] {
        for seed in 0..100 {
            let map = MapConfig {
                seed,
                width: 32,
                height: 32,
                generator: family,
                wall_percent: 0,
                stress: Some(Default::default()),
            };
            let grid = generate(&map)?;
            if grid != generate(&map)? {
                return Err("map mismatch".into());
            }
            let mut logs = Vec::new();
            for a in [
                Algorithm::Bfs,
                Algorithm::Dfs,
                Algorithm::Dijkstra,
                Algorithm::Astar,
                Algorithm::Greedy,
            ] {
                let cfg = RunConfig::new(
                    a,
                    if matches!(a, Algorithm::Astar | Algorithm::Greedy) {
                        Heuristic::Manhattan
                    } else {
                        Heuristic::Zero
                    },
                );
                let first = execute(&grid, &map, &cfg)?;
                let second = execute(&grid, &map, &cfg)?;
                if first.log.canonical_bytes() != second.log.canonical_bytes() {
                    return Err("event mismatch".into());
                }
                hash_checks += 1;
                seeks += verify_execution(&first, 10)?;
                seeks += verify_execution(&second, 10)?;
                logs.push(first.log);
            }
            let b = &logs[0].metrics;
            let d = &logs[2].metrics;
            let best = logs
                .iter()
                .filter(|l| l.config.algorithm != Algorithm::Dfs)
                .map(|l| l.metrics.expanded)
                .min()
                .unwrap();
            let dfs = logs[1].metrics.expanded as f64 / best as f64;
            let greedy = logs[4]
                .metrics
                .path_cost
                .zip(d.path_cost)
                .map(|(a, b)| a as f64 / b as f64);
            let astar = logs[3].metrics.expanded as f64 / d.expanded as f64;
            let bfs_cost = b
                .path_cost
                .zip(d.path_cost)
                .map(|(a, b)| a as f64 / b as f64);
            let hit = match family {
                Generator::DfsDeepTrap => {
                    dfs >= arena_core::observation::thresholds()["dfs"]["moderate_ratio"]
                        .as_f64()
                        .unwrap()
                }
                Generator::GreedyTrap => greedy.is_some_and(|r| r > 1.0),
                Generator::LowInformation => {
                    astar
                        >= arena_core::observation::thresholds()["astar"]["strong_expansion_ratio"]
                            .as_f64()
                            .unwrap()
                }
                Generator::BfsWeightedLesson => bfs_cost.is_some_and(|r| r > 1.0),
                _ => false,
            };
            if family == Generator::BfsWeightedLesson && b.path_length > d.path_length {
                return Err("BFS step invariant".into());
            }
            rows.push(json!({"family":family,"seed":seed,"effect_hit":hit,"dfs_expansion_ratio":dfs,"greedy_cost_ratio":greedy,"greedy_expanded":logs[4].metrics.expanded,"dijkstra_expanded":d.expanded,"astar_expansion_ratio":astar,"bfs_cost_ratio":bfs_cost,"bfs_steps":b.path_length,"dijkstra_steps":d.path_length,"hashes":logs.iter().map(|l|&l.canonical_hash).collect::<Vec<_>>()}));
        }
    }
    Ok(
        json!({"version":"stress-corpus/1","seeds_per_family":100,"maps":400,"runs":4000,"hash_comparisons":hash_checks,"seek_checks":seeks,"map_mismatch":0,"hash_mismatch":0,"replay_divergence":0,"thresholds":arena_core::observation::thresholds(),"records":rows}),
    )
}
fn percentile(values: &mut [f64], p: usize) -> f64 {
    values.sort_by(f64::total_cmp);
    values[((values.len() - 1) * p / 100).min(values.len() - 1)]
}
pub fn profile() -> Result<Value, String> {
    // Exactly the existing 128 corpus: 8 families × 3 seeds × 5 algorithms.
    let families = [
        ("open", Generator::Open, 0),
        ("sparse", Generator::Obstacles, 15),
        ("dense", Generator::Obstacles, 35),
        ("maze", Generator::Backtracker, 0),
        ("corridors", Generator::Corridors, 0),
        ("weighted", Generator::Weighted, 15),
        ("rooms", Generator::Rooms, 0),
        ("heuristic_trap", Generator::HeuristicTrap, 0),
    ];
    let mut records = Vec::new();
    for (family, generator, wall_percent) in families {
        for seed in [123, 456, 789] {
            let map = MapConfig {
                seed,
                width: 128,
                height: 128,
                generator,
                wall_percent,
                stress: None,
            };
            let grid = generate(&map)?;
            for a in [
                Algorithm::Bfs,
                Algorithm::Dijkstra,
                Algorithm::Astar,
                Algorithm::Dfs,
                Algorithm::Greedy,
            ] {
                let cfg = RunConfig::new(
                    a,
                    if matches!(a, Algorithm::Astar | Algorithm::Greedy) {
                        Heuristic::Manhattan
                    } else {
                        Heuristic::Zero
                    },
                );
                let run = execute(&grid, &map, &cfg)?;
                let events = &run.log.events;
                let serialization = Instant::now();
                let serialized = serde_json::to_vec(&run.log).map_err(|e| e.to_string())?;
                let serialization_ms = serialization.elapsed().as_secs_f64() * 1000.0;
                let mut rng = arena_core::grid::SeededRng::new(seed);
                let targets = (0..100)
                    .map(|_| rng.below((events.len() + 1) as u64) as usize)
                    .collect::<Vec<_>>();
                for budget in [32, 128] {
                    let build = Instant::now();
                    let mut replay = Replay::with_budget(events, grid.cells.len(), budget);
                    let build_ms = build.elapsed().as_secs_f64() * 1000.0;
                    let memory = replay.checkpoint_count()
                        * grid.cells.len()
                        * std::mem::size_of::<arena_core::replay::NodeState>();
                    let mut times = Vec::new();
                    let mut suffix = 0;
                    for target in &targets {
                        let at = Instant::now();
                        replay.seek(*target)?;
                        times.push(at.elapsed().as_secs_f64() * 1000.0);
                        suffix += target % replay.checkpoint_interval;
                    }
                    let mut expected = ReplayState::new(grid.cells.len());
                    for event in events {
                        expected.apply(event);
                    }
                    replay.seek(events.len())?;
                    if replay.state != expected {
                        return Err("profile replay divergence".into());
                    }
                    records.push(json!({"family":family,"seed":seed,"algorithm":a,"strategy":if budget==32{"fixed32"}else{"density128"},"events":events.len(),"hash":run.log.canonical_hash,"checkpoint_count":replay.checkpoint_count(),"checkpoint_interval":replay.checkpoint_interval,"average_suffix":suffix as f64/targets.len()as f64,"checkpoint_memory_bytes_proxy":memory,"serialization_ms":serialization_ms,"serialized_bytes":serialized.len(),"reconstruction_build_ms":build_ms,"seek_samples_ms":times.clone(),"seek_p50_ms":percentile(&mut times,50),"seek_p95_ms":percentile(&mut times,95)}));
                }
            }
        }
    }
    Ok(
        json!({"version":"seek-profile/1","corpus":"unchanged Phase 1 128 corpus, 120 runs, 100 identical targets per strategy","memory_definition":"checkpoint count × nodes × size_of NodeState; excludes allocator, paths, log","strategies":["fixed32 max(256,events/32)","density128 max(256,events/128)"],"records":records}),
    )
}

pub fn benchmark_stress() -> Result<Value, String> {
    let mut records = Vec::new();
    for size in [32, 64, 128] {
        for generator in [
            Generator::DfsDeepTrap,
            Generator::GreedyTrap,
            Generator::LowInformation,
            Generator::BfsWeightedLesson,
        ] {
            for seed in [123, 456, 789] {
                let params = arena_core::stress::StressParams {
                    branch_count: 8,
                    branch_depth: size - 8,
                    goal_placement: size - 4,
                    ..Default::default()
                };
                let map = MapConfig {
                    seed,
                    width: size,
                    height: size,
                    generator,
                    wall_percent: 0,
                    stress: Some(params),
                };
                let grid = generate(&map)?;
                for a in [
                    Algorithm::Bfs,
                    Algorithm::Dijkstra,
                    Algorithm::Astar,
                    Algorithm::Dfs,
                    Algorithm::Greedy,
                ] {
                    let config = RunConfig::new(
                        a,
                        if matches!(a, Algorithm::Astar | Algorithm::Greedy) {
                            Heuristic::Manhattan
                        } else {
                            Heuristic::Zero
                        },
                    );
                    let log = execute(&grid, &map, &config)?.log;
                    let m = &log.metrics;
                    records.push(json!({"mapFamily":generator,"algorithm":a,"size":size,"seed":seed,"expanded":m.expanded,"logicalOps":m.logical_ops,"pathCost":m.path_cost,"peakFrontier":m.peak_frontier,"computeMs":log.runtime.compute_ns as f64/1e6,"events":log.events.len(),"runHash":log.canonical_hash,"mapConfig":map}));
                }
            }
        }
    }
    Ok(
        json!({"engineVersion":arena_core::ENGINE_VERSION,"generatorVersion":"stress/1","build":if cfg!(debug_assertions){"debug"}else{"release"},"os":std::env::consts::OS,"arch":std::env::consts::ARCH,"timingScope":"search + event collection; excludes map generation, hash and playback","records":records}),
    )
}
