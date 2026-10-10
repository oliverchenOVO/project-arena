use arena_core::sorting::{self, SortConfig};
use serde_json::{json, Value};
fn check_replay(log: &arena_core::domain::DomainLog) -> Result<usize, String> {
    use arena_core::domain_replay::{DomainReplay, State};
    let mut replay = DomainReplay::new(log);
    let mut linear = State::new(log);
    let targets = [
        0,
        log.events.len() / 4,
        log.events.len() / 2,
        log.events.len(),
    ];
    for (i, e) in log.events.iter().enumerate() {
        if targets.contains(&i) {
            replay.seek(i)?;
            if replay.state != linear {
                return Err("checkpoint divergence".into());
            }
        }
        linear.apply(e);
    }
    replay.seek(log.events.len())?;
    if replay.state != linear {
        return Err("terminal replay divergence".into());
    }
    linear.check_metrics(log)?;
    Ok(targets.len())
}
pub fn benchmark(domain: &str) -> Result<Value, String> {
    if !["sorting", "graph", "all"].contains(&domain) {
        return Err("domain must be pathfinding, sorting, graph or all".into());
    }
    let mut records = vec![];
    let mut repeat = 0;
    let mut seeks = 0;
    if domain == "sorting" || domain == "all" {
        for family in sorting::FAMILIES.iter().take(6) {
            for size in [16, 32, 64, 128, 256] {
                for seed in 0..5 {
                    let config = SortConfig {
                        length: size,
                        distribution: (*family).into(),
                        seed,
                        ..Default::default()
                    };
                    let a = sorting::generate(&config)?;
                    let mut expected = a.clone();
                    expected.sort();
                    for algorithm in sorting::ALGORITHMS {
                        let log = sorting::execute(&a, algorithm, &config)?;
                        let second = sorting::execute(&a, algorithm, &config)?;
                        if log.canonical_hash != second.canonical_hash {
                            return Err("sorting determinism mismatch".into());
                        }
                        repeat += 1;
                        let replay = sorting::replay(&log, log.events.len())?;
                        if replay != expected {
                            return Err("sorting correctness or replay mismatch".into());
                        }
                        seeks += check_replay(&log)?;
                        let m = &log.metrics;
                        records.push(json!({"domain":"sorting","algorithm":algorithm,"family":family,"size":size,"seed":seed,"comparisons":m["comparisons"],"swaps":m["swaps"],"writes":m["writes"],"reads":m["reads"],"logicalOps":m["logicalOps"],"auxiliaryMemory":m["auxiliaryMemory"],"computeMs":log.compute_ns as f64/1e6,"events":log.events.len(),"runHash":log.canonical_hash}));
                    }
                }
            }
        }
    }
    if domain == "graph" || domain == "all" {
        for family in [
            "sparse",
            "dense",
            "tree",
            "clustered",
            "bridge",
            "disconnected",
        ] {
            for size in [32, 64, 128, 256] {
                for seed in 0..5 {
                    let c = arena_core::graph::GraphConfig {
                        size,
                        seed,
                        family: family.into(),
                        directed: false,
                        weighted: true,
                    };
                    let graph = arena_core::graph::generate(&c)?;
                    let mut mst = None;
                    let mut cost = None;
                    for algorithm in arena_core::graph::ALGORITHMS {
                        if family == "disconnected" && matches!(algorithm, "prim" | "kruskal") {
                            continue;
                        }
                        let log = arena_core::graph::execute(&graph, algorithm)?;
                        if log.canonical_hash
                            != arena_core::graph::execute(&graph, algorithm)?.canonical_hash
                        {
                            return Err("graph determinism mismatch".into());
                        }
                        repeat += 1;
                        let m = &log.metrics;
                        seeks += check_replay(&log)?;
                        if matches!(algorithm, "prim" | "kruskal") {
                            if mst.is_some_and(|v| v != m["totalWeight"]) {
                                return Err("MST weight mismatch".into());
                            }
                            mst = Some(m["totalWeight"].clone());
                        }
                        if matches!(algorithm, "dijkstra" | "astar") {
                            if cost.is_some_and(|v| v != m["pathCost"]) {
                                return Err("shortest path mismatch".into());
                            }
                            cost = Some(m["pathCost"].clone());
                        }
                        records.push(json!({"domain":"graph","algorithm":algorithm,"family":family,"size":size,"seed":seed,"visited":m["visited"],"edgesExamined":m["edgesExamined"],"totalWeight":m["totalWeight"],"frontierPeak":m["frontierPeak"],"dsuOps":m["dsuOps"],"priorityQueueOps":m["priorityQueueOps"],"pathCost":m["pathCost"],"pathLength":m["pathLength"],"selectedEdges":m["selectedEdges"],"rejectedEdges":m["rejectedEdges"],"logicalOps":m["logicalOps"],"computeMs":log.compute_ns as f64/1e6,"events":log.events.len(),"runHash":log.canonical_hash}));
                    }
                }
            }
        }
    }
    Ok(
        json!({"version":2,"domain":domain,"records":records,"determinismComparisons":repeat,"determinismMismatches":0,"replayChecks":seeks,"replayDivergences":0,"timingScope":"algorithm + event recording; excludes generation and hashing"}),
    )
}
