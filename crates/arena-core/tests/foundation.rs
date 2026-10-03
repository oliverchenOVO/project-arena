use arena_core::event::EventKind;
use arena_core::grid::SeededRng;
use arena_core::replay::{Replay, ReplayState};
use arena_core::validation::{reference_cost, verify_execution};
use arena_core::{
    execute, generate, Algorithm, Generator, Grid, Heuristic, MapConfig, RunConfig, Terrain,
};

fn cfg(a: Algorithm) -> RunConfig {
    RunConfig::new(
        a,
        if a == Algorithm::Astar {
            Heuristic::Manhattan
        } else {
            Heuristic::Zero
        },
    )
}
fn problem(grid: &Grid) -> MapConfig {
    MapConfig {
        width: grid.width,
        height: grid.height,
        ..MapConfig::default()
    }
}
fn open(w: usize, h: usize) -> Grid {
    generate(&MapConfig {
        width: w,
        height: h,
        generator: Generator::Open,
        ..MapConfig::default()
    })
    .unwrap()
}

#[test]
fn rng_reproducibility_and_known_vector() {
    let mut a = SeededRng::new(0);
    assert_eq!(a.next_u64(), 0xe220a8397b1dcdaf);
    let (mut a, mut b) = (SeededRng::new(123), SeededRng::new(123));
    for _ in 0..1000 {
        assert_eq!(a.next_u64(), b.next_u64());
        assert_eq!(a.below(17), b.below(17));
    }
}
#[test]
fn generators_are_deterministic_across_dimensions() {
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
        for (width, height) in [
            (1, 1),
            (1, 8),
            (8, 1),
            (2, 2),
            (15, 16),
            (32, 32),
            (128, 128),
        ] {
            for seed in 0..5 {
                let map = MapConfig {
                    generator,
                    width,
                    height,
                    seed,
                    ..MapConfig::default()
                };
                assert_eq!(generate(&map).unwrap(), generate(&map).unwrap());
            }
        }
    }
}
#[test]
fn neighbor_order_is_north_east_south_west() {
    assert_eq!(open(3, 3).neighbors(4), vec![1, 5, 7, 3]);
}
#[test]
fn terrain_costs_are_centralized() {
    assert_eq!(
        [
            Terrain::Normal.cost(),
            Terrain::Mud.cost(),
            Terrain::Water.cost(),
            Terrain::Heavy.cost(),
            Terrain::Wall.cost()
        ],
        [Some(1), Some(3), Some(5), Some(10), None]
    );
}
#[test]
fn bfs_uniform_grid_is_optimal() {
    for seed in 0..20 {
        let map = MapConfig {
            seed,
            width: 12,
            height: 12,
            ..MapConfig::default()
        };
        let grid = generate(&map).unwrap();
        let run = execute(&grid, &map, &cfg(Algorithm::Bfs)).unwrap();
        assert_eq!(run.log.metrics.path_length, reference_cost(&grid, true));
        verify_execution(&run, 8).unwrap();
    }
}
#[test]
fn dijkstra_weighted_matches_independent_oracle() {
    for seed in 0..20 {
        let map = MapConfig {
            seed,
            width: 10,
            height: 10,
            generator: Generator::Weighted,
            ..MapConfig::default()
        };
        let grid = generate(&map).unwrap();
        let run = execute(&grid, &map, &cfg(Algorithm::Dijkstra)).unwrap();
        assert_eq!(run.log.metrics.path_cost, reference_cost(&grid, false));
    }
}
#[test]
fn all_admissible_astar_heuristics_match_dijkstra() {
    for seed in 0..20 {
        let map = MapConfig {
            seed,
            width: 15,
            height: 15,
            generator: Generator::Weighted,
            ..MapConfig::default()
        };
        let grid = generate(&map).unwrap();
        let d = execute(&grid, &map, &cfg(Algorithm::Dijkstra)).unwrap();
        for h in [
            Heuristic::Manhattan,
            Heuristic::Euclidean,
            Heuristic::Chebyshev,
            Heuristic::Zero,
        ] {
            let a = execute(&grid, &map, &RunConfig::new(Algorithm::Astar, h)).unwrap();
            assert_eq!(a.log.metrics.path_cost, d.log.metrics.path_cost);
        }
    }
}
#[test]
fn heuristic_consistency_on_every_neighbor() {
    let grid = open(23, 19);
    for h in [
        Heuristic::Manhattan,
        Heuristic::Euclidean,
        Heuristic::Chebyshev,
        Heuristic::Zero,
    ] {
        assert_eq!(h.estimate(&grid, grid.goal), 0);
        for n in 0..grid.cells.len() {
            for next in grid.neighbors(n) {
                assert!(h.estimate(&grid, n) <= 1 + h.estimate(&grid, next));
            }
        }
    }
}
#[test]
fn bfs_does_not_claim_weighted_optimality() {
    let mut grid = open(3, 2);
    grid.goal = 2;
    grid.cells[1] = Terrain::Heavy;
    let b = execute(&grid, &problem(&grid), &cfg(Algorithm::Bfs)).unwrap();
    let d = execute(&grid, &problem(&grid), &cfg(Algorithm::Dijkstra)).unwrap();
    assert_eq!(b.log.metrics.path_cost, Some(11));
    assert_eq!(d.log.metrics.path_cost, Some(4));
}
#[test]
fn start_equals_goal() {
    let grid = open(1, 1);
    for a in [Algorithm::Bfs, Algorithm::Dijkstra, Algorithm::Astar] {
        let run = execute(&grid, &problem(&grid), &cfg(a)).unwrap();
        assert_eq!(run.path, vec![0]);
        assert_eq!(run.log.metrics.path_cost, Some(0));
        assert_eq!(run.log.metrics.path_length, Some(0));
        verify_execution(&run, 4).unwrap();
    }
}
#[test]
fn blocked_goal_and_disconnected_regions() {
    let mut grid = open(3, 3);
    grid.cells[5] = Terrain::Wall;
    grid.cells[7] = Terrain::Wall;
    for a in [Algorithm::Bfs, Algorithm::Dijkstra, Algorithm::Astar] {
        let run = execute(&grid, &problem(&grid), &cfg(a)).unwrap();
        assert!(run.path.is_empty());
        assert_eq!(run.log.metrics.path_cost, None);
        verify_execution(&run, 4).unwrap();
    }
}
#[test]
fn one_cell_corridor() {
    let grid = open(1, 64);
    let run = execute(&grid, &problem(&grid), &cfg(Algorithm::Astar)).unwrap();
    assert_eq!(run.log.metrics.path_cost, Some(63));
}
#[test]
fn all_high_cost_terrain() {
    let mut grid = open(8, 8);
    grid.cells.fill(Terrain::Heavy);
    for a in [Algorithm::Bfs, Algorithm::Dijkstra, Algorithm::Astar] {
        let run = execute(&grid, &problem(&grid), &cfg(a)).unwrap();
        assert_eq!(run.log.metrics.path_cost, Some(140));
        verify_execution(&run, 4).unwrap();
    }
}
#[test]
fn larger_open_grid() {
    let grid = open(128, 128);
    for a in [Algorithm::Bfs, Algorithm::Dijkstra, Algorithm::Astar] {
        let run = execute(&grid, &problem(&grid), &cfg(a)).unwrap();
        assert_eq!(run.log.metrics.path_cost, Some(254));
        verify_execution(&run, 4).unwrap();
    }
}
#[test]
fn priority_tie_break_is_heuristic_then_insertion() {
    let grid = open(3, 3);
    let d = execute(&grid, &problem(&grid), &cfg(Algorithm::Dijkstra)).unwrap();
    let a = execute(&grid, &problem(&grid), &cfg(Algorithm::Astar)).unwrap();
    let expanded = |events: &[arena_core::event::Event]| {
        events
            .iter()
            .filter_map(|e| {
                if let EventKind::NodeExpanded { node } = e.kind {
                    Some(node)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(expanded(&d.log.events), vec![0, 1, 3, 2, 4, 6, 5, 7, 8]);
    assert_eq!(expanded(&a.log.events), vec![0, 1, 2, 5, 8]);
}
#[test]
fn canonical_log_stable_and_runtime_excluded() {
    let grid = open(10, 10);
    let map = problem(&grid);
    let a = execute(&grid, &map, &cfg(Algorithm::Astar)).unwrap();
    let mut b = execute(&grid, &map, &cfg(Algorithm::Astar)).unwrap();
    b.log.runtime.compute_ns += 999;
    assert_eq!(a.log.canonical_bytes(), b.log.canonical_bytes());
    assert_eq!(a.log.hash(), b.log.hash());
}
#[test]
fn event_serialization_roundtrip_and_ordering() {
    let grid = open(8, 8);
    let run = execute(&grid, &problem(&grid), &cfg(Algorithm::Dijkstra)).unwrap();
    let bytes = serde_json::to_vec(&run.log).unwrap();
    let restored: arena_core::RunLog = serde_json::from_slice(&bytes).unwrap();
    restored.validate().unwrap();
    assert_eq!(restored.canonical_bytes(), run.log.canonical_bytes());
    for (i, e) in restored.events.iter().enumerate() {
        assert_eq!(e.sequence, i);
        if i > 0 {
            assert!(e.tick >= restored.events[i - 1].tick);
        }
    }
}
#[test]
fn metric_accounting_is_exact() {
    let grid = open(2, 1);
    let run = execute(&grid, &problem(&grid), &cfg(Algorithm::Bfs)).unwrap();
    let m = &run.log.metrics;
    assert_eq!(
        (
            m.expanded,
            m.generated,
            m.edges_evaluated,
            m.frontier_pushes,
            m.frontier_pops,
            m.cost_updates,
            m.relaxation_comparisons
        ),
        (2, 2, 1, 2, 2, 2, 1)
    );
    assert_eq!(m.logical_ops, 8);
    assert_eq!(
        m.logical_ops,
        m.expanded
            + m.edges_evaluated
            + m.frontier_pushes
            + m.frontier_pops
            + m.relaxation_comparisons
    );
    assert_eq!(m.peak_frontier, 1);
    assert_eq!(m.peak_discovered, 2);
    assert_eq!(m.peak_state_entries, 6);
}
#[test]
fn replay_seek_and_step_are_equivalent_to_prefix() {
    let grid = open(12, 12);
    let run = execute(&grid, &problem(&grid), &cfg(Algorithm::Dijkstra)).unwrap();
    verify_execution(&run, 100).unwrap();
    let mut replay = Replay::new(&run.log.events, grid.cells.len());
    assert!(replay.step_expansion());
    assert_eq!(replay.state.accounting.metrics.expanded, 1);
    replay.seek(0).unwrap();
    assert!(!replay.step_backward());
    assert!(replay.seek(run.log.events.len() + 1).is_err());
    let mut prefix = ReplayState::new(grid.cells.len());
    for event in &run.log.events {
        prefix.apply(event);
        replay.step_forward();
        assert_eq!(prefix, replay.state);
    }
}
#[test]
fn path_reconstruction_validates_every_edge_and_cost() {
    let grid = open(3, 3);
    assert_eq!(grid.path_cost(&[0, 1, 2, 5, 8]), Ok(4));
    assert!(grid.path_cost(&[0, 8]).is_err());
    assert!(grid.path_cost(&[0, 999, 8]).is_err());
    assert!(grid.path_cost(&[]).is_err());
}
#[test]
fn malformed_configuration_is_rejected() {
    for (width, height, wall_percent) in [(0, 1, 0), (1, 0, 0), (257, 1, 0), (1, 1, 101)] {
        assert!(generate(&MapConfig {
            width,
            height,
            wall_percent,
            ..MapConfig::default()
        })
        .is_err());
    }
    let mut grid = open(2, 2);
    grid.start = 999;
    assert!(execute(&grid, &problem(&grid), &cfg(Algorithm::Bfs)).is_err());
    let mut grid = open(2, 2);
    grid.cells[0] = Terrain::Wall;
    assert!(grid.validate().is_err());
    let grid = open(2, 2);
    let mut bad = cfg(Algorithm::Bfs);
    bad.neighbor_order = "random".into();
    assert!(execute(&grid, &problem(&grid), &bad).is_err());
    assert!(execute(
        &grid,
        &problem(&grid),
        &RunConfig::new(Algorithm::Dijkstra, Heuristic::Manhattan)
    )
    .is_err());
}
#[test]
fn corrupted_import_is_rejected_before_replay() {
    let grid = open(3, 3);
    let run = execute(&grid, &problem(&grid), &cfg(Algorithm::Astar)).unwrap();
    let mut log = run.log.clone();
    log.events[1].kind = EventKind::NodeDiscovered { node: 999 };
    assert!(log.validate().is_err());
    log.canonical_hash = log.hash();
    assert!(log.validate().is_err());
    let mut log = run.log;
    log.events[1].kind = EventKind::FrontierRemoved { node: 0 };
    log.canonical_hash = log.hash();
    assert!(log.validate().is_err());
}
#[test]
fn deterministic_features_have_defined_denominators() {
    let grid = open(3, 3);
    let f = arena_core::analysis::features(&grid, &[0, 1, 2, 5, 8]);
    assert_eq!(f.obstacle_density, 0.0);
    assert_eq!(f.open_space_ratio, 1.0);
    assert_eq!(f.dead_end_count, 0);
    assert_eq!(f.path_tortuosity, Some(1.0));
    assert!(arena_core::analysis::features(&open(1, 1), &[0])
        .path_tortuosity
        .is_none());
}
