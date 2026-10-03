use arena_core::grid::SeededRng;
use arena_core::replay::Replay;
use arena_core::validation::{reference_cost, verify_execution};
use arena_core::{
    execute, generate, Algorithm, Generator, Heuristic, MapConfig, RunConfig, RunLog,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::time::Instant;

fn algorithm(value: &str) -> Result<Algorithm, String> {
    match value {
        "bfs" => Ok(Algorithm::Bfs),
        "dijkstra" => Ok(Algorithm::Dijkstra),
        "astar" => Ok(Algorithm::Astar),
        _ => Err(format!("unknown algorithm: {value}")),
    }
}
fn generator(value: &str) -> Result<Generator, String> {
    match value {
        "open" => Ok(Generator::Open),
        "obstacles" => Ok(Generator::Obstacles),
        "backtracker" | "maze" => Ok(Generator::Backtracker),
        "division" => Ok(Generator::Division),
        "rooms" => Ok(Generator::Rooms),
        "corridors" => Ok(Generator::Corridors),
        "weighted" => Ok(Generator::Weighted),
        "heuristic_trap" => Ok(Generator::HeuristicTrap),
        _ => Err(format!("unknown generator: {value}")),
    }
}
fn heuristic(value: &str) -> Result<Heuristic, String> {
    match value {
        "manhattan" => Ok(Heuristic::Manhattan),
        "euclidean" => Ok(Heuristic::Euclidean),
        "chebyshev" => Ok(Heuristic::Chebyshev),
        "zero" => Ok(Heuristic::Zero),
        _ => Err(format!("unknown heuristic: {value}")),
    }
}
fn config(a: Algorithm) -> RunConfig {
    RunConfig::new(
        a,
        if a == Algorithm::Astar {
            Heuristic::Manhattan
        } else {
            Heuristic::Zero
        },
    )
}
fn output(path: Option<&String>, value: &Value) -> Result<(), String> {
    if let Some(path) = path {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        println!("Report: {path}");
    }
    Ok(())
}

fn gate(options: &BTreeMap<String, String>) -> Result<(), String> {
    let began = Instant::now();
    let families = [
        Generator::Open,
        Generator::Obstacles,
        Generator::Backtracker,
        Generator::Division,
        Generator::Rooms,
        Generator::Corridors,
        Generator::Weighted,
        Generator::HeuristicTrap,
    ];
    let (mut executions, mut seek_cases, mut replay_cases, mut optimal_cases) = (0, 0, 0, 0);
    for seed in 0..100 {
        let map = MapConfig {
            seed,
            width: 16,
            height: 16,
            generator: families[seed as usize % families.len()],
            wall_percent: if seed % 2 == 0 { 15 } else { 35 },
        };
        let grid = generate(&map)?;
        if grid != generate(&map)? {
            return Err(format!("map determinism mismatch seed {seed}"));
        }
        let cost_oracle = reference_cost(&grid, false);
        let steps_oracle = reference_cost(&grid, true);
        for a in [Algorithm::Bfs, Algorithm::Dijkstra, Algorithm::Astar] {
            let first = execute(&grid, &map, &config(a))?;
            let second = execute(&grid, &map, &config(a))?;
            executions += 2;
            if first.log.canonical_bytes() != second.log.canonical_bytes()
                || first.log.canonical_hash != second.log.canonical_hash
            {
                return Err(format!("run determinism mismatch {seed} {a:?}"));
            }
            for run in [&first, &second] {
                seek_cases += verify_execution(run, 10)?;
                replay_cases += 1;
                let actual = if a == Algorithm::Bfs {
                    run.log.metrics.path_length
                } else {
                    run.log.metrics.path_cost
                };
                let expected = if a == Algorithm::Bfs {
                    steps_oracle
                } else {
                    cost_oracle
                };
                if actual != expected {
                    return Err(format!(
                        "optimality mismatch {seed} {a:?}: {actual:?} != {expected:?}"
                    ));
                }
                optimal_cases += 1;
            }
        }
        if seed % 20 == 19 {
            println!("Validated seeds 0..{seed} ({executions} executions)");
        }
    }
    let report = json!({"status":"PHASE 0 PASS","engineVersion":arena_core::ENGINE_VERSION,"seedsTested":100,"algorithms":["bfs","dijkstra","astar"],"repetitions":2,"executions":executions,"generatorFamilies":8,"mapDeterminismCases":100,"determinismComparisons":300,"determinismMismatches":0,"invalidPaths":0,"crashes":0,"replayCases":replay_cases,"seekCases":seek_cases,"replayDivergences":0,"optimalityCases":optimal_cases,"durationMs":began.elapsed().as_secs_f64()*1000.0});
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
    output(options.get("output"), &report)
}

fn percentile(values: &mut [u64], numerator: usize) -> f64 {
    values.sort_unstable();
    let index = (values.len() * numerator)
        .div_ceil(100)
        .saturating_sub(1)
        .min(values.len() - 1);
    values[index] as f64 / 1e6
}
fn benchmark(options: &BTreeMap<String, String>) -> Result<(), String> {
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
    for size in [32, 64, 128] {
        for (family, generator, wall_percent) in families {
            for seed in [123, 456, 789] {
                let map = MapConfig {
                    seed,
                    width: size,
                    height: size,
                    generator,
                    wall_percent,
                };
                let grid = generate(&map)?;
                for a in [Algorithm::Bfs, Algorithm::Dijkstra, Algorithm::Astar] {
                    let run = execute(&grid, &map, &config(a))?;
                    let began = Instant::now();
                    let mut replay = Replay::new(&run.log.events, grid.cells.len());
                    let checkpoint_ms = began.elapsed().as_secs_f64() * 1000.0;
                    let began = Instant::now();
                    while replay.step_forward() {}
                    let replay_ms = began.elapsed().as_secs_f64() * 1000.0;
                    let mut rng = SeededRng::new(seed);
                    let mut latencies = Vec::new();
                    for _ in 0..50 {
                        let target = rng.below((run.log.events.len() + 1) as u64) as usize;
                        let began = Instant::now();
                        replay.seek(target)?;
                        latencies.push(began.elapsed().as_nanos() as u64);
                    }
                    let m = &run.log.metrics;
                    records.push(json!({"seed":seed,"mapFamily":family,"size":size,"algorithm":a,"expanded":m.expanded,"generated":m.generated,"logicalOps":m.logical_ops,"peakFrontier":m.peak_frontier,"stateFootprintProxy":m.peak_state_entries,"pathLength":m.path_length,"pathCost":m.path_cost,"computeMs":run.log.runtime.compute_ns as f64/1e6,"events":run.log.events.len(),"checkpointBuildMs":checkpoint_ms,"checkpointInterval":replay.checkpoint_interval,"replayMs":replay_ms,"seekSamples":50,"seekP50Ms":percentile(&mut latencies,50),"seekP95Ms":percentile(&mut latencies,95),"runHash":run.log.canonical_hash}));
                }
            }
        }
        println!("Measured {size}×{size}: {} records so far", records.len());
    }
    let report = json!({"engineVersion":arena_core::ENGINE_VERSION,"build":"release recommended; caller must record actual profile","timingScope":"search + event collection; excludes hashing/map generation/playback","records":records});
    output(options.get("output"), &report)?;
    if options.get("output").is_none() {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    }
    Ok(())
}

fn main_run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "help".into());
    let mut options = BTreeMap::new();
    while let Some(key) = args.next() {
        if !key.starts_with("--") {
            return Err(format!("expected --option, got {key}"));
        }
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for {key}"))?;
        if options.insert(key[2..].to_string(), value).is_some() {
            return Err(format!("duplicate option {key}"));
        }
    }
    let allowed: &[&str] = match command.as_str() {
        "run" => &[
            "seed",
            "width",
            "height",
            "generator",
            "walls",
            "algorithm",
            "heuristic",
            "output",
        ],
        "gate" | "benchmark" => &["output"],
        "replay" => &["input", "cursor"],
        "help" => &[],
        _ => return Err(format!("unknown command {command}")),
    };
    for key in options.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(format!("unknown option --{key}"));
        }
    }
    match command.as_str() {
        "gate" => gate(&options),
        "benchmark" => benchmark(&options),
        "run" => {
            let mut map = MapConfig::default();
            if let Some(v) = options.get("seed") {
                map.seed = v.parse().map_err(|_| "invalid seed")?;
            }
            if let Some(v) = options.get("width") {
                map.width = v.parse().map_err(|_| "invalid width")?;
            }
            if let Some(v) = options.get("height") {
                map.height = v.parse().map_err(|_| "invalid height")?;
            }
            if let Some(v) = options.get("walls") {
                map.wall_percent = v.parse().map_err(|_| "invalid walls")?;
            }
            if let Some(v) = options.get("generator") {
                map.generator = generator(v)?;
            }
            let a = algorithm(
                options
                    .get("algorithm")
                    .map(String::as_str)
                    .unwrap_or("astar"),
            )?;
            let mut cfg = config(a);
            if let Some(v) = options.get("heuristic") {
                cfg.heuristic = heuristic(v)?;
            }
            let grid = generate(&map)?;
            let execution = execute(&grid, &map, &cfg)?;
            execution.log.validate()?;
            println!("Algorithm: {a:?}\nSeed: {}\nExpanded: {}\nGenerated: {}\nPath cost: {:?}\nPeak frontier: {}\nLogical ops: {}\nEvents: {}\nCompute: {:.3} ms\nRun hash: {}",map.seed,execution.log.metrics.expanded,execution.log.metrics.generated,execution.log.metrics.path_cost,execution.log.metrics.peak_frontier,execution.log.metrics.logical_ops,execution.log.events.len(),execution.log.runtime.compute_ns as f64/1e6,execution.log.canonical_hash);
            let value = serde_json::to_value(&execution.log).map_err(|e| e.to_string())?;
            output(options.get("output"), &value)
        }
        "replay" => {
            let path = options.get("input").ok_or("replay needs --input")?;
            let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
            let log: RunLog = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            log.validate()?;
            let cursor = options
                .get("cursor")
                .map(|v| v.parse::<usize>())
                .transpose()
                .map_err(|_| "invalid cursor")?
                .unwrap_or(log.events.len());
            let mut replay = Replay::new(&log.events, log.grid.cells.len());
            replay.seek(cursor)?;
            println!(
                "Replayed {cursor}/{} saved events; completed: {}; metrics: {:?}",
                log.events.len(),
                replay.state.completed,
                replay.state.accounting.metrics
            );
            Ok(())
        }
        _ => {
            println!("ARENA headless\n  arena run --seed 123 --algorithm astar --generator weighted --output run.arena.json\n  arena replay --input run.arena.json --cursor 256\n  arena gate --output phase0-gate.json\n  arena benchmark --output benchmark.json");
            Ok(())
        }
    }
}
fn main() {
    if let Err(error) = main_run() {
        eprintln!("ARENA error: {error}");
        std::process::exit(1);
    }
}
