//! The same application service is used by Tauri and the browser dev adapter.
use crate::{execute, generate, Grid, MapConfig, RunConfig, RunLog};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    DomainGenerate {
        domain: crate::domain::ArenaDomain,
        config: Value,
    },
    DomainRace {
        domain: crate::domain::ArenaDomain,
        problem: Value,
        algorithms: Vec<String>,
    },
    DomainValidate {
        logs: Vec<crate::domain::DomainLog>,
    },
    Generate {
        map_config: MapConfig,
    },
    Race {
        grid: Grid,
        map_config: MapConfig,
        configs: Vec<RunConfig>,
    },
    Validate {
        logs: Vec<RunLog>,
    },
}
pub fn handle(request: Request) -> Result<Value, String> {
    match request {
        Request::DomainGenerate { domain, config } => match domain {
            crate::domain::ArenaDomain::Sorting => {
                let c: crate::sorting::SortConfig =
                    serde_json::from_value(config).map_err(|e| e.to_string())?;
                Ok(json!({"problem":{"values":crate::sorting::generate(&c)?,"config":c}}))
            }
            crate::domain::ArenaDomain::Graph => {
                let c = serde_json::from_value(config).map_err(|e| e.to_string())?;
                Ok(json!({"problem":crate::graph::generate(&c)?}))
            }
            _ => Err("domain generator unavailable".into()),
        },
        Request::DomainRace {
            domain,
            problem,
            algorithms,
        } => {
            if algorithms.is_empty() || algorithms.len() > 4 {
                return Err("race needs 1..4 contenders".into());
            }
            let mut logs = vec![];
            for a in algorithms {
                logs.push(domain_execute(domain, &problem, &a)?);
            }
            Ok(domain_result(&logs))
        }
        Request::DomainValidate { logs } => {
            if logs.is_empty() || logs.len() > 4 {
                return Err("import needs 1..4 logs".into());
            }
            for log in &logs {
                log.validate_envelope()?;
                if log.domain != logs[0].domain || log.problem != logs[0].problem {
                    return Err("race logs must share the same problem".into());
                }
                let expected = domain_execute(log.domain, &log.problem, &log.algorithm_id)?;
                if log.canonical_hash != expected.canonical_hash {
                    return Err("v2 execution mismatch".into());
                }
            }
            Ok(domain_result(&logs))
        }
        Request::Generate { map_config } => Ok(json!({"grid":generate(&map_config)?})),
        Request::Race {
            grid,
            map_config,
            configs,
        } => {
            let lab = configs.len() == 5
                && configs[4].algorithm == crate::Algorithm::Dijkstra
                && configs[..4].iter().all(|c| {
                    c.algorithm == configs[0].algorithm
                        && matches!(
                            c.algorithm,
                            crate::Algorithm::Astar | crate::Algorithm::Greedy
                        )
                })
                && [
                    crate::Heuristic::Zero,
                    crate::Heuristic::Manhattan,
                    crate::Heuristic::Euclidean,
                    crate::Heuristic::Chebyshev,
                ]
                .iter()
                .all(|h| configs[..4].iter().any(|c| c.heuristic == *h));
            if configs.is_empty() || (configs.len() > 4 && !lab) {
                return Err("race needs 1..4 contenders".into());
            }
            // Desktop bounds limit log volume, not algorithm correctness.
            if grid.width > 128 || grid.height > 128 {
                return Err("interactive grids limited to 128×128".into());
            }
            let mut logs = Vec::new();
            let mut features = Vec::new();
            for config in configs {
                let run = execute(&grid, &map_config, &config)?;
                features.push(crate::analysis::features(&grid, &run.path));
                logs.push(run.log);
            }
            Ok(result(&logs, &features))
        }
        Request::Validate { logs } => {
            if logs.is_empty() || logs.len() > 5 {
                return Err("import needs 1..4 logs".into());
            }
            let first = &logs[0];
            for log in &logs {
                log.validate()?;
                if log.grid != first.grid || log.map_config != first.map_config {
                    return Err("race logs must share the same problem".into());
                }
            }
            let features = logs
                .iter()
                .map(|log| {
                    let path = log
                        .events
                        .iter()
                        .find_map(|e| {
                            if let crate::event::EventKind::PathReconstructed { path, .. } = &e.kind
                            {
                                Some(path.as_slice())
                            } else {
                                None
                            }
                        })
                        .unwrap_or(&[]);
                    crate::analysis::features(&log.grid, path)
                })
                .collect::<Vec<_>>();
            Ok(result(&logs, &features))
        }
    }
}

fn domain_execute(
    domain: crate::domain::ArenaDomain,
    problem: &Value,
    a: &str,
) -> Result<crate::domain::DomainLog, String> {
    match domain {
        crate::domain::ArenaDomain::Sorting => {
            let values: Vec<i64> =
                serde_json::from_value(problem["values"].clone()).map_err(|e| e.to_string())?;
            let config =
                serde_json::from_value(problem["config"].clone()).map_err(|e| e.to_string())?;
            crate::sorting::execute(&values, a, &config)
        }
        crate::domain::ArenaDomain::Graph => {
            let graph = serde_json::from_value(problem.clone()).map_err(|e| e.to_string())?;
            crate::graph::execute(&graph, a)
        }
        _ => Err("domain execution unavailable".into()),
    }
}

fn domain_result(logs: &[crate::domain::DomainLog]) -> Value {
    let problem = &logs[0].problem;
    let features = if logs[0].domain == crate::domain::ArenaDomain::Sorting {
        crate::sorting::features(
            &serde_json::from_value::<Vec<i64>>(problem["values"].clone()).unwrap_or_default(),
        )
    } else {
        json!({"nodeCount":problem["nodes"].as_array().map(Vec::len),"edgeCount":problem["edges"].as_array().map(Vec::len),"layout":"seeded integer grid jitter / v1"})
    };
    let observations = crate::observe::observe(crate::observe::RunSet::Domain(logs));
    json!({"logs":logs,"features":observations["derived"],"observations":observations,"problemFeatures":features})
}

fn result(logs: &[RunLog], features: &[crate::analysis::MapFeatures]) -> Value {
    let baseline = logs
        .iter()
        .find(|l| l.config.algorithm == crate::Algorithm::Dijkstra);
    let steps = logs
        .iter()
        .find(|l| l.config.algorithm == crate::Algorithm::Bfs)
        .and_then(|l| l.metrics.path_length);
    let observations = crate::observation::observations(logs, baseline, steps);
    let markers = logs
        .iter()
        .map(crate::observation::markers)
        .collect::<Vec<_>>();
    json!({"logs":logs,"features":features,"observations":observations,"observationV2":crate::observe::observe(crate::observe::RunSet::Pathfinding(logs)),"markers":markers})
}
