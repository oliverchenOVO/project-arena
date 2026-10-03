use std::process::Command;

fn cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_arena")).args(args).output().unwrap()
}
#[test]
fn help_describes_headless_commands() {
    let result=cli(&["help"]);assert!(result.status.success());assert!(String::from_utf8_lossy(&result.stdout).contains("arena replay"));
}
#[test]
fn unknown_and_duplicate_options_are_errors() {
    for args in [vec!["run","--algo","astar"],vec!["run","--seed","1","--seed","2"],vec!["run","--seed"],vec!["invalid"]] {assert!(!cli(&args).status.success());}
}
#[test]
fn invalid_problem_and_algorithm_are_errors() {
    for args in [vec!["run","--width","0"],vec!["run","--algorithm","unknown"],vec!["run","--algorithm","dijkstra","--heuristic","manhattan"]] {assert!(!cli(&args).status.success());}
}
#[test]
fn saved_run_replays_without_reexecution() {
    let path=std::env::temp_dir().join(format!("arena-cli-test-{}.arena.json",std::process::id()));
    let filename=path.to_str().unwrap();
    let run=cli(&["run","--width","5","--height","5","--generator","open","--output",filename]);assert!(run.status.success());
    let replay=cli(&["replay","--input",filename]);assert!(replay.status.success());assert!(String::from_utf8_lossy(&replay.stdout).contains("completed: true"));
    let replay=cli(&["replay","--input",filename,"--cursor","0"]);assert!(replay.status.success());assert!(String::from_utf8_lossy(&replay.stdout).contains("completed: false"));
    assert!(!cli(&["replay","--input",filename,"--cursor","999999"]).status.success());std::fs::remove_file(path).unwrap();
}
