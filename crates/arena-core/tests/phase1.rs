use arena_core::validation::verify_execution;
use arena_core::{execute, generate, Algorithm, Generator, Heuristic, MapConfig, RunConfig};

#[test]
fn dfs_and_greedy_terminate_with_valid_paths_on_all_families() {
    for generator in [
        Generator::Open,
        Generator::Obstacles,
        Generator::Backtracker,
        Generator::Division,
        Generator::Rooms,
        Generator::Corridors,
        Generator::Weighted,
        Generator::HeuristicTrap,
    ] {
        for seed in 0..10 {
            let map = MapConfig {
                seed,
                width: 16,
                height: 16,
                generator,
                ..MapConfig::default()
            };
            let grid = generate(&map).unwrap();
            for algorithm in [Algorithm::Dfs, Algorithm::Greedy] {
                let config = RunConfig::new(
                    algorithm,
                    if algorithm == Algorithm::Greedy {
                        Heuristic::Manhattan
                    } else {
                        Heuristic::Zero
                    },
                );
                let run = execute(&grid, &map, &config).unwrap();
                assert!(run.log.metrics.expanded <= grid.cells.len() as u64);
                if !run.path.is_empty() {
                    assert_eq!(
                        grid.path_cost(&run.path).unwrap(),
                        run.log.metrics.path_cost.unwrap()
                    );
                }
                verify_execution(&run, 5).unwrap();
            }
        }
    }
}
#[test]
fn service_rejects_incompatible_race_imports_and_empty_contenders() {
    use arena_core::bridge::{handle, Request};
    let map = MapConfig::default();
    let grid = generate(&map).unwrap();
    assert!(handle(Request::Race {
        grid: grid.clone(),
        map_config: map.clone(),
        configs: vec![]
    })
    .is_err());
    let config = RunConfig::new(Algorithm::Astar, Heuristic::Manhattan);
    let a = execute(&grid, &map, &config).unwrap();
    let map2 = MapConfig { seed: 99, ..map };
    let b = execute(&generate(&map2).unwrap(), &map2, &config).unwrap();
    assert!(handle(Request::Validate {
        logs: vec![a.log, b.log]
    })
    .is_err());
}
