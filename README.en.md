# ARENA — AI Algorithm Arena

An execution-driven multi-domain algorithm experiment platform built with Rust, React, TypeScript, Vite and Tauri 2. Compare algorithms on the same problem, inspect recorded state and replay the exact event log offline.

[中文作品介紹](README.md) · [繁體中文操作指南](README.zh-TW.md) · Version **0.2.0**

![Actual packaged A* vs Dijkstra replay](docs/screenshots/phase12-zh-astar-dijkstra-race.png)

## Phase 2 multi-domain arena

Pathfinding (7 implementations), Sorting (6), and Graph (6) share Rust execution, canonical event logs, pure replay reducers, inspectors and Observer v2. Sorting includes six real instrumented algorithms and seven seeded distributions; Graph includes eight deterministic-layout generators, traversal, shortest paths and MST comparisons. Weighted A* supports weights 1/1.25/1.5/2/3; bidirectional BFS requires symmetric unit-cost movement.

900 sorting records and 680 graph records each undergo repeated hash comparison and four checkpoint checks: zero mismatches and zero divergences. The 2,540 historical hash comparisons remain unchanged. Current validation: 46 Rust, 61 frontend and 43 E2E tests. See [acceptance](docs/phase2.md), [design](docs/phase2-design.md), [summary](phase2-summary.json), and [actual Windows verification](windows-phase2-verification.json).

![Actual packaged Prim versus Kruskal](docs/screenshots/phase2-zh-graph-mst.png)

## Historical Phase 1.2 localization and presentation

Traditional Chinese and English share a complete central dictionary. The saved preference wins over browser language, with zh-TW as the fallback. Switching language keeps the same map, seed, logs, metrics, playback position and canonical export. No search runs during a language switch.

Start with five real fixed-seed showcase cards or enter the laboratory. The guided demo pauses at three recorded milestones, continues to measured analysis and repeats the same hashes on restart. Configuration collapses after execution; Presentation Mode keeps race panes, metrics and playback in focus and exits with Escape. Analysis shows an observed headline and four independent metric charts, with measured data, derived formulas and rule-based interpretation identified separately.

See [Phase 1.2 acceptance](docs/phase12.md), [machine-readable evidence](phase12-summary.json) and [Windows verification](windows-phase12-verification.json). All 540 legacy benchmark records and 2,000 saved stress hashes remain identical.

## Phase 1.1 laboratory

Version **0.1.1** added four Rust stress generators, EXPOSE WEAKNESS, four-heuristic comparison with a measured Dijkstra reference, Ghost Overlay / Diff Mode, recorded node history, replay markers, three synchronization modes, rule-based observations, five Showcase presets / Demo Mode and a saved-JSON BENCHMARK viewer.

See [Phase 1.1 evidence and contracts](docs/phase11.md) and [measured summary](phase11-summary.json). The unchanged 360-record corpus reproduces every previous hash/event count; the new four-family corpus has 4,000 repeated executions and zero divergences. A denser checkpoint candidate was measured on the same 128 corpus and rejected; the original strategy remains.

![Packaged Ghost comparison](docs/screenshots/phase11-windows-ghost.png)

## Product

- **BUILD:** seeded generators, dimensions, start/goal placement, drag painting, weighted terrain, erase and clear. Editing invalidates the previous run.
- **RACE:** configurable 1v1 or four contenders, shared event clock and Canvas rendering. Default Dijkstra vs A*.
- **INSPECT:** real recorded status, distance, h/f where applicable, parent and expansion sequence.
- **ANALYZE:** measured comparisons, deterministic map features and structured explanation input.
- **Replay:** play/pause, event forward/backward, expansion step, seek, beginning/end, 0.25×–8× and MAX; `.arena.json` import/export.
- **Headless:** execution, canonical hashes, saved-log replay, gates and benchmark JSON/readable summaries.
- **Windows:** x64 executable and unsigned NSIS installer; actual packaged execution, replay, inspection and analysis verified with HTTPS requests blocked.

No API key is required. Execution never calls an LLM. The optional explanation provider interface is isolated; the unconfigured provider is a muted informational panel. No live AI provider is configured.

## Architecture

```mermaid
flowchart LR
    C[Seed + config] --> G[Rust Grid generator]
    G --> E[Rust execution]
    E --> L[Canonical events / RunLog]
    L --> P[Checkpoint replay]
    P --> V[React Canvas / Inspector]
    L --> A[Deterministic features]
    A --> X[Optional explanation provider]
    L --> H[Headless validation]
```

`arena-core` owns generation, execution, events, accounting, validation and analysis. `arena-cli` is headless. Tauri commands call the same application service on a blocking worker. The Vite development adapter invokes the Rust CLI; it has no fake-data fallback or TypeScript search implementation. `src/playback.ts` knows only the event contract. Canvas subscribes to batched playback updates, without per-event grid allocations in React.

Execution finishes into a log before visual playback starts. Playback duration never enters runtime metrics.

## Algorithms

| Algorithm | Frontier | Guarantee |
| --- | --- | --- |
| BFS | FIFO | Minimum steps; weighted cost can differ |
| Dijkstra | Priority queue | Optimal non-negative terrain cost |
| A* | Priority queue, g + h | Optimal cost with supported admissible heuristics |
| DFS | LIFO | Reachability; no shortest-path guarantee |
| Greedy | Priority queue, h | Reachability; no shortest-path guarantee |

Four-direction neighbors: N → E → S → W. DFS emits neighbors in that order and explores the last eligible pushed node first. Heap ties prefer lower priority, lower h, earlier insertion. Manhattan, integer-floor Euclidean, Chebyshev and zero are all admissible/consistent with four-direction movement and minimum cost 1.

Destination terrain costs: normal **1**, mud **3**, water **5**, heavy **10**; walls are impassable. Start costs zero. Start may equal goal. Unreachable path cost/length is null, not zero.

Generators: open, obstacles, recursive backtracker, recursive division, rooms, corridors, weighted terrain and a heuristic stress map. Disconnected random maps are not silently repaired. Interactive dimensions: 1–128; headless: 1–256.

## Determinism and replay

SplitMix64 v1 is the sole seeded RNG, using explicit wrapping arithmetic and unbiased bounded sampling. Logs record grid, seed, generator config/version, RNG/version, algorithm/version, heuristic, neighbor/tie-break rules, terrain costs and engine/schema versions. For edited maps, the saved grid is authoritative.

Canonical SHA-256 covers compact ordered serialization of deterministic inputs, metadata, events and algorithmic metrics. Runtime is excluded. Schema 1 uses typed events with event sequence and expansion tick; immutable identity/configuration is log-level. The UI supports safe JavaScript integer seeds; the CLI accepts full u64 seeds.

Rust validates imported versions, hashes, paths, event ordering and accounting; race logs must share the exact problem. Replay reads saved events without search. Checkpoints use at least 256 events, increasing the interval to cap full-state snapshots at 33. Race playback uses one event-count clock: 1× = 1,000 events/s per contender. Step Expansion advances to the next expansion in any contender. See [execution contracts](docs/contracts.md).

## Metrics

Expanded/discovered nodes, evaluated traversable edges, frontier pushes/pops, comparisons, cost updates, peak frontier, final path edges/cost and event counts are measured from execution.

**ARENA Logical Operations** = expansion + edge evaluation + queue push + queue pop + relaxation comparison, each +1. This is not CPU instructions. Stale heap pops count as pops, not expansions.

**State Footprint Proxy** counts peak active discovered/cost/parent/frontier entries, not RAM or bytes. Compute measures search plus event collection/accounting; generation, hashing, serialization and playback are outside that interval. Events/sec uses compute duration.

Analysis separates measured metrics, derived obstacle/open-space/corridor/dead-end/weighted ratios and path tortuosity, and explicitly labeled hypotheses. Expansion ratio to Dijkstra appears in the structured summary; no undefined overall score is used.

## Development and checks

Prerequisites: Node 24+, Rust stable, Visual Studio C++ build tools for Windows/MSVC, and WebView2 Runtime for desktop.

```powershell
npm ci
cargo build --release -p arena-cli
npm run dev                    # http://127.0.0.1:1431
npm run desktop:dev
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --release
npm run lint
npm run typecheck
npm test
npm run build
npx playwright install chromium
npm run test:e2e
```

If installed Rust is absent from PATH, prepend `$env:USERPROFILE\.cargo\bin`. Browser mode needs the Rust dev adapter; `vite preview` serves assets only. Use Tauri for standalone offline execution.

Rust: **39 tests**; frontend: **48 unit cases**, including replay reduction against a real Rust-produced fixture and unreachable-cost analysis. **33 Playwright flows** cover three pairings × five map families, editor validity, four contenders, saved-log import and 390px responsiveness, five showcase presets, live locale persistence, canonical export invariance and English core smoke. The original 24 flows now run in zh-TW; nine added flows cover localization and both languages. Gates use an independent O(V²) optimality oracle, with separate step/cost objectives.

```powershell
cargo run --release -p arena-cli -- gate --output phase0-gate.json
cargo run --release -p arena-cli -- gate-phase1 --output phase1-core-gate.json
```

Phase 0: 100 seeds × 3 algorithms × 2 repeats = **600 runs**, 600 replays, 16,950 seeks. Phase 1 core: **1,000 runs/replays**, **24,966 seeks**. Both had zero determinism mismatches, invalid paths, crashes or replay divergence. GitHub Actions verifies Rust, frontend and Windows build; consult the latest run before claiming remote CI is green.

## CLI and real benchmark

```powershell
cargo run --release -p arena-cli -- run --seed 123 --algorithm astar --generator weighted --output example.arena.json
cargo run --release -p arena-cli -- replay --input example.arena.json --cursor 256
cargo run --release -p arena-cli -- benchmark --output benchmark.json
```

8 families × seeds 123/456/789 × 32/64/128 × 5 algorithms = **360 records**, with 50 seek samples each. Actual Windows x64 release samples, aggregating each size's 120 runs:

| Grid | Compute p50 ms | Compute p95 ms | Highest run seek p95 ms |
| --- | ---: | ---: | ---: |
| 32×32 | 0.637 | 3.898 | 0.398 |
| 64×64 | 2.494 | 39.292 | 1.672 |
| 128×128 | 8.677 | 277.989 | 252.207 |

These are non-isolated local measurements across different problems, including unreachable maps, not a universal speed ranking. `benchmark.json` retains compute times, event counts, checkpoint construction, replay and seek p50/p95. Large-log seek latency/memory remain a tradeoff; the controlled Phase 1.1 comparison is documented separately.

## Windows build

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --check
npm run desktop:build
```

Outputs: `src-tauri/target/release/project-arena.exe` and `src-tauri/target/release/bundle/nsis/Project ARENA_0.1.2_x64-setup.exe`. Builds generate and include locked dependency license notices. The installer can download WebView2 when missing; offline core works once the runtime is installed. Binaries are unsigned. Code signing and clean-machine installer testing are not claimed.

![Localized recorded node inspector](docs/screenshots/phase12-zh-inspector-astar-dijkstra.png)
![Localized measured analysis](docs/screenshots/phase12-zh-analysis.png)

## Roadmap

Continue investigating compact replay states, configure an optional explanation provider and sign Windows releases. Sorting, 3D, accounts, multiplayer and extra algorithm families stay outside Phase 1 scope.

MIT license. See `THIRD_PARTY_NOTICES.md` and `docs/acceptance.md` for limitations and evidence.
