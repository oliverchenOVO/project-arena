use arena_core::observation::{diff, history, markers, observations, thresholds};
use arena_core::replay::{Replay, ReplayState};
use arena_core::stress::StressParams;
use arena_core::{execute, generate, Algorithm, Generator, Heuristic, MapConfig, RunConfig};
fn map(g: Generator, seed: u64) -> MapConfig {
    MapConfig {
        generator: g,
        seed,
        width: 32,
        height: 32,
        wall_percent: 0,
        stress: Some(StressParams::default()),
    }
}
#[test]
fn legacy_saved_fixture_keeps_its_canonical_hash() {
    let log: arena_core::RunLog = serde_json::from_str(include_str!(
        "../../../src/__tests__/fixtures/weighted-run.json"
    ))
    .unwrap();
    log.validate().unwrap();
    let repeated = execute(&log.grid, &log.map_config, &log.config).unwrap();
    assert_eq!(repeated.log.canonical_hash, log.canonical_hash);
}
#[test]
fn zero_heuristic_lab_has_zero_h_and_matches_dijkstra() {
    let m = map(Generator::LowInformation, 123);
    let grid = generate(&m).unwrap();
    let a = execute(
        &grid,
        &m,
        &RunConfig::new(Algorithm::Astar, Heuristic::Zero),
    )
    .unwrap()
    .log;
    let d = run(&m, Algorithm::Dijkstra).log;
    assert_eq!(a.metrics, d.metrics);
    let o = observations(&[a], Some(&d), None);
    assert_eq!(o["derived"][0]["average_h_at_expansion"], 0.0);
    assert_eq!(o["derived"][0]["heuristic_guidance_ratio"], 0.0);
}
fn run(m: &MapConfig, a: Algorithm) -> arena_core::engine::Execution {
    execute(
        &generate(m).unwrap(),
        m,
        &RunConfig::new(
            a,
            if matches!(a, Algorithm::Astar | Algorithm::Greedy) {
                Heuristic::Manhattan
            } else {
                Heuristic::Zero
            },
        ),
    )
    .unwrap()
}
#[test]
fn four_stress_generators_repeat_maps_hashes_and_replay() {
    for g in [
        Generator::DfsDeepTrap,
        Generator::GreedyTrap,
        Generator::LowInformation,
        Generator::BfsWeightedLesson,
    ] {
        for seed in 0..100 {
            let m = map(g, seed);
            assert_eq!(generate(&m).unwrap(), generate(&m).unwrap());
            let a = run(&m, Algorithm::Astar);
            let b = run(&m, Algorithm::Astar);
            assert_eq!(a.log.canonical_hash, b.log.canonical_hash);
            arena_core::validation::verify_execution(&a, 3).unwrap();
        }
    }
}
#[test]
fn weighted_lesson_preserves_bfs_step_objective() {
    for seed in 0..100 {
        let m = map(Generator::BfsWeightedLesson, seed);
        let b = run(&m, Algorithm::Bfs);
        let d = run(&m, Algorithm::Dijkstra);
        assert!(b.log.metrics.path_length <= d.log.metrics.path_length);
        assert!(b.log.metrics.path_cost > d.log.metrics.path_cost);
    }
}
#[test]
fn parameters_and_orientation_are_deterministic_and_validated() {
    for orientation in 0..4 {
        let mut m = map(Generator::DfsDeepTrap, 123);
        m.stress.as_mut().unwrap().orientation = orientation;
        assert_eq!(generate(&m).unwrap(), generate(&m).unwrap());
    }
    let mut m = map(Generator::DfsDeepTrap, 0);
    m.stress.as_mut().unwrap().corridor_width = 0;
    assert!(generate(&m).is_err());
    m.stress.as_mut().unwrap().corridor_width = 1;
    m.stress.as_mut().unwrap().version = "stress/999".into();
    assert!(generate(&m).is_err());
}
#[test]
fn observations_use_measured_counts_and_public_thresholds() {
    let m = map(Generator::DfsDeepTrap, 0);
    let d = run(&m, Algorithm::Dijkstra).log;
    let dfs = run(&m, Algorithm::Dfs).log;
    let a = run(&m, Algorithm::Astar).log;
    let logs = vec![dfs, a, d.clone()];
    let o = observations(&logs, Some(&d), None);
    assert_eq!(thresholds()["version"], "stress-effect/1");
    assert_eq!(
        o["measured"][0]["metrics"]["expanded"],
        logs[0].metrics.expanded
    );
    assert_eq!(o["derived"][0]["badge"], "STRONGLY EXPOSED");
    assert_eq!(o["derived"][2]["expansion_ratio_to_dijkstra"], 1.0);
    assert!(o["derived"][1]["path_cost_optimal"].as_bool().unwrap());
    assert!(o["derived"][1]["average_h_at_expansion"].as_f64().unwrap() >= 0.0);
    assert!(o["interpretation"][0]["rule_version"].is_string());
}
#[test]
fn missing_reference_and_unreachable_do_not_invent_ratios() {
    let m = map(Generator::DfsDeepTrap, 1);
    let d = run(&m, Algorithm::Dfs).log;
    let o = observations(&[d], None, None);
    assert!(o["derived"][0]["expansion_ratio_to_dijkstra"].is_null());
    assert!(o["derived"][0]["path_cost_optimal"].is_null());
    assert!(arena_core::observation::ratio(1, 0).is_none());
}
#[test]
fn ghost_history_and_markers_match_real_prefixes() {
    let m = map(Generator::BfsWeightedLesson, 4);
    let a = run(&m, Algorithm::Bfs).log;
    let b = run(&m, Algorithm::Dijkstra).log;
    let mut left = Replay::new(&a.events, a.grid.cells.len());
    let mut right = Replay::new(&b.events, b.grid.cells.len());
    assert_eq!(diff(&left.state, &right.state)["shared_expanded"], 0);
    left.seek(a.events.len()).unwrap();
    right.seek(b.events.len()).unwrap();
    let value = diff(&left.state, &right.state);
    assert_eq!(
        value["shared_expanded"].as_u64().unwrap() + value["left_only"].as_u64().unwrap(),
        a.metrics.expanded
    );
    assert_eq!(
        value["shared_expanded"].as_u64().unwrap() + value["right_only"].as_u64().unwrap(),
        b.metrics.expanded
    );
    for marker in markers(&a) {
        let c = marker["cursor"].as_u64().unwrap() as usize;
        assert!(c > 0 && c <= a.events.len());
    }
    let h = history(&a, a.grid.start, a.events.len());
    assert!(!h["events"].as_array().unwrap().is_empty());
    assert_eq!(h["parent_changes"], 0);
    assert!(history(&a, a.grid.goal, 0)["events"]
        .as_array()
        .unwrap()
        .is_empty());
}
#[test]
fn alternative_checkpoints_preserve_every_sampled_prefix() {
    let m = map(Generator::GreedyTrap, 10);
    let a = run(&m, Algorithm::Dijkstra);
    let mut expected = ReplayState::new(a.log.grid.cells.len());
    let mut old = Replay::with_budget(&a.log.events, a.log.grid.cells.len(), 32);
    let mut dense = Replay::with_budget(&a.log.events, a.log.grid.cells.len(), 128);
    for (i, event) in a.log.events.iter().enumerate() {
        expected.apply(event);
        if i % 123 == 0 || i + 1 == a.log.events.len() {
            old.seek(i + 1).unwrap();
            dense.seek(i + 1).unwrap();
            assert_eq!(old.state, expected);
            assert_eq!(dense.state, expected);
        }
    }
}
