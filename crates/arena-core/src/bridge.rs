//! The same application service is used by Tauri and the browser dev adapter.
use crate::{execute, generate, Grid, MapConfig, RunConfig, RunLog};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
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
        Request::Generate { map_config } => Ok(json!({"grid":generate(&map_config)?})),
        Request::Race {
            grid,
            map_config,
            configs,
        } => {
            if configs.is_empty() || configs.len() > 4 {
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
            Ok(json!({"logs":logs,"features":features}))
        }
        Request::Validate { logs } => {
            if logs.is_empty() || logs.len() > 4 {
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
            Ok(json!({"logs":logs,"features":features}))
        }
    }
}
