use arena_core::domain_replay::{DomainReplay, State};
#[test]
fn advanced_pathfinding_preserves_objectives_and_weighted_priority() {
    use arena_core::{execute, generate, Algorithm, Generator, Heuristic, MapConfig, RunConfig};
    for seed in 0..50 {
        let c = MapConfig {
            width: 16,
            height: 16,
            seed,
            generator: Generator::Obstacles,
            wall_percent: 25,
            stress: None,
        };
        let g = generate(&c).unwrap();
        let bfs = execute(&g, &c, &RunConfig::new(Algorithm::Bfs, Heuristic::Zero)).unwrap();
        let bidir = execute(
            &g,
            &c,
            &RunConfig::new(Algorithm::BidirectionalBfs, Heuristic::Zero),
        )
        .unwrap();
        assert_eq!(bfs.log.metrics.path_cost, bidir.log.metrics.path_cost);
        arena_core::validation::verify_execution(&bidir, 10).unwrap();
        let d = execute(
            &g,
            &c,
            &RunConfig::new(Algorithm::Dijkstra, Heuristic::Zero),
        )
        .unwrap();
        for w in [100, 125, 150, 200, 300] {
            let mut rc = RunConfig::new(Algorithm::WeightedAstar, Heuristic::Manhattan);
            rc.weight_percent = Some(w);
            let run = execute(&g, &c, &rc).unwrap();
            run.log.validate().unwrap();
            assert_eq!(
                run.log.canonical_hash,
                execute(&g, &c, &rc).unwrap().log.canonical_hash
            );
            if w == 100 {
                assert_eq!(run.log.metrics.path_cost, d.log.metrics.path_cost);
            }
            for e in &run.log.events {
                if let arena_core::event::EventKind::WeightedPriority {
                    node,
                    scaled_f,
                    scale,
                } = e.kind
                {
                    assert_eq!(scale, 100);
                    assert_eq!(
                        scaled_f % 100,
                        (rc.heuristic.estimate(&g, node) * w as u64) % 100
                    );
                }
            }
        }
    }
    let c = MapConfig {
        generator: Generator::Weighted,
        ..Default::default()
    };
    let g = generate(&c).unwrap();
    assert!(execute(
        &g,
        &c,
        &RunConfig::new(Algorithm::BidirectionalBfs, Heuristic::Zero)
    )
    .is_err());
}
use arena_core::graph::{self, GraphConfig};
use arena_core::sorting::{self, SortConfig};

#[test]
fn graph_generation_and_layout_determinism() {
    for family in graph::FAMILIES {
        for directed in [false, true] {
            let c = GraphConfig {
                size: 64,
                seed: 123,
                family: family.into(),
                directed,
                weighted: true,
            };
            let g = graph::generate(&c).unwrap();
            assert_eq!(g, graph::generate(&c).unwrap());
            g.validate().unwrap();
        }
    }
}
#[test]
fn graph_shortest_path_matches_independent_bellman_ford() {
    for family in graph::FAMILIES {
        for seed in 0..8 {
            let g = graph::generate(&GraphConfig {
                size: 32,
                seed,
                family: family.into(),
                directed: seed % 2 == 0,
                weighted: true,
            })
            .unwrap();
            let mut d = vec![u64::MAX; g.nodes.len()];
            d[0] = 0;
            for _ in 1..g.nodes.len() {
                for e in &g.edges {
                    if d[e.from] != u64::MAX {
                        d[e.to] = d[e.to].min(d[e.from] + e.weight)
                    }
                    if !g.directed && d[e.to] != u64::MAX {
                        d[e.from] = d[e.from].min(d[e.to] + e.weight)
                    }
                }
            }
            let expected = if d[g.goal] == u64::MAX {
                None
            } else {
                Some(d[g.goal])
            };
            for a in ["dijkstra", "astar"] {
                let log = graph::execute(&g, a).unwrap();
                assert_eq!(log.metrics["pathCost"].as_u64(), expected);
            }
        }
    }
}
#[test]
fn mst_traversal_and_replay_contracts() {
    for family in graph::FAMILIES {
        let g = graph::generate(&GraphConfig {
            size: 32,
            seed: 42,
            family: family.into(),
            directed: false,
            weighted: true,
        })
        .unwrap();
        let mut weight = None;
        for a in graph::ALGORITHMS {
            if family == "disconnected" && matches!(a, "prim" | "kruskal") {
                assert!(graph::execute(&g, a).is_err());
                continue;
            }
            let log = graph::execute(&g, a).unwrap();
            assert_eq!(
                log.canonical_hash,
                graph::execute(&g, a).unwrap().canonical_hash
            );
            if matches!(a, "prim" | "kruskal") {
                assert_eq!(log.metrics["selectedEdges"], 31);
                if let Some(v) = weight {
                    assert_eq!(v, log.metrics["totalWeight"])
                }
                weight = Some(log.metrics["totalWeight"].clone());
            }
            if matches!(a, "bfs" | "dfs") {
                assert_eq!(log.metrics["visited"], 32)
            }
            let mut replay = DomainReplay::new(&log);
            assert!(replay.checkpoints.len() <= 33);
            let mut state = State::new(&log);
            for i in 0..=log.events.len() {
                replay.seek(i).unwrap();
                assert_eq!(replay.state, state);
                if i < log.events.len() {
                    state.apply(&log.events[i]);
                }
            }
            state.check_metrics(&log).unwrap();
        }
    }
}
#[test]
fn sorting_correctness_determinism_accounting_replay() {
    for input in [
        vec![],
        vec![1],
        vec![4, 3, 2, 1],
        vec![2, 1, 2, 1, 0],
        vec![-2, -5, 0, 8],
    ] {
        for a in sorting::ALGORITHMS {
            let c = SortConfig::default();
            let log = sorting::execute(&input, a, &c).unwrap();
            log.validate_envelope().unwrap();
            let mut expected = input.clone();
            expected.sort();
            assert_eq!(sorting::replay(&log, log.events.len()).unwrap(), expected);
            assert_eq!(
                log.canonical_hash,
                sorting::execute(&input, a, &c).unwrap().canonical_hash
            );
            assert_eq!(
                log.metrics["comparisons"].as_u64().unwrap(),
                log.events
                    .iter()
                    .filter(|e| e.event_type == "Compare")
                    .count() as u64
            );
            assert_eq!(
                log.metrics["reads"].as_u64().unwrap(),
                log.events.iter().filter(|e| e.event_type == "Read").count() as u64
            );
            assert_eq!(
                log.metrics["writes"].as_u64().unwrap(),
                log.events
                    .iter()
                    .filter(|e| e.event_type == "Write")
                    .count() as u64
                    + 2 * log.events.iter().filter(|e| e.event_type == "Swap").count() as u64
            );
            for cursor in 0..=log.events.len() {
                assert_eq!(sorting::replay(&log, cursor).unwrap().len(), input.len());
            }
        }
    }
}
#[test]
fn stable_sort_identity_order() {
    for a in ["bubble", "insertion", "merge"] {
        let log = sorting::execute(&[2, 1, 2, 1, 2], a, &SortConfig::default()).unwrap();
        assert_eq!(
            log.events.last().unwrap().payload["identities"],
            serde_json::json!([1, 3, 0, 2, 4])
        );
    }
}
#[test]
fn sorting_generators_are_seeded_and_bounded() {
    for family in sorting::FAMILIES {
        let c = SortConfig {
            length: 256,
            distribution: family.into(),
            ..Default::default()
        };
        let a = sorting::generate(&c).unwrap();
        assert_eq!(a, sorting::generate(&c).unwrap());
        assert!(a.iter().all(|v| *v >= c.min && *v <= c.max));
    }
}
