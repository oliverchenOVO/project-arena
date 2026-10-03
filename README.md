# Project ARENA — AI Algorithm Arena

A deterministic algorithm experiment platform. Real search execution emits a versioned event log; playback consumes that log without running an algorithm again.

## Current milestone

Phase 0 implementation in progress. No UI or desktop packaging is claimed yet. The acceptance report will be generated from actual headless checks.

## Architecture

```
Seed + MapConfig → Rust Grid → Search → Canonical Events + Metrics
                                      ↓
                                  Saved RunLog
                                      ↓
                              Checkpointed Replay
                                      ↓
                            Future React Canvas UI
```

`arena-core` has no UI or Tauri dependency. `arena-cli` runs experiments, replay validation and benchmarks. React/TypeScript/Vite/Tauri 2 are the planned Phase 1 presentation and desktop layers.

## Development

Install a Rust MSVC toolchain and Visual Studio C++ build tools on Windows.

```
cargo run -p arena-cli -- run --seed 123 --algorithm astar
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```

See `docs/contracts.md` for deterministic execution and metric definitions.

