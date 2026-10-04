# Phase 1.1 — Showcase Intelligence & Stress Lab

Local acceptance evidence for Project ARENA 0.1.1, Windows x64, 2026-10-04. All local functional, determinism, regression and packaged checks passed. Code commit `081e4a2` passed [Rust/frontend/Windows CI](https://github.com/oliverchenOVO/project-arena/actions/runs/37182219080) and the [100-seed stress gate](https://github.com/oliverchenOVO/project-arena/actions/runs/37182219052). Final-head remote CI status is reported in the delivery message; workflows run on every pushed commit. No new search algorithm or execution ordering policy was introduced.

## IMPLEMENTED

- EXPOSE WEAKNESS selects DFS, Greedy, A*, Dijkstra or BFS and executes a Rust-generated seeded scenario. Dijkstra uses the low-information layout for comparison with A*.
- Four `stress/1` families, reproducible generation, canonical exports/imports, headless corpus and benchmark commands. These are stress tendencies, without a mathematically worst-case claim.
- Heuristic laboratory executes Zero, Manhattan, integer-floor Euclidean and Chebyshev on the same grid, plus a fifth measured Dijkstra reference. Ordinary races retain 1–4 contenders; the service permits five only for this laboratory configuration.
- Single selected Canvas field: expansion ordinal, g, h, g+h, sampled/local parent tree or 3×3 frontier density (fraction of nearby cells with a positive recorded frontier-entry count). g is available for Dijkstra/A*; h/f for A*/Greedy. Parent arrows are capped at 400, with a selected cell's neighborhood preferred.
- Split View / Overlay Diff / Metrics. Ghost counts use the current recorded replay states. Shared expansions are gray, left-only teal, right-only blue; shared visited cells have a muted fill. White/amber/pink distinguish shared and divergent recorded paths.
- Inspector event history, exact event IDs, actual parent changes (initial assignment excluded), cross-algorithm comparison and history jumps. Future/absent events are omitted.
- Clickable timeline diamonds and accessible marker buttons for goal discovery, goal expansion, reconstructed path, 25/50/75% cost-update milestones and first maximum frontier moment.
- Event Index, Expansion Count and Percentage synchronization. Expansion mode compares the same expansion count and caps a shorter run at its final expansion; end restores every completed log. Marker jumps preserve the exact selected log event and synchronize the other logs by the selected rule.
- Scientific RACE COMPLETE comparison and Rust `observation/1` output, explicitly separated into MEASURED, DERIVED and INTERPRETATION.
- Five fixed Showcase presets, real execution and Demo → replay → analysis. Restart produces identical hashes.
- BENCHMARK is a saved-JSON viewer: family/algorithm heatmap, paired-seed A* reduction, expanded/ops/cost/frontier aggregates and compute p50/p95. It does not launch searches.
- Vendor-neutral optional explanation payload contains problem metadata, measured values, derived values and deterministic interpretation. No provider is configured; the observation engine remains available.

## Stress generator contract

`MapConfig.stress` is optional and omitted from serialization when absent. All previous canonical hashes remain stable. New families use generator version `stress/1`; SplitMix64, destination terrain costs, N/E/S/W neighbor ordering, priority/h/insertion tie breaks and algorithm version 1 remain unchanged.

DFS defaults: branch_count=8, branch_depth=24, goal_placement=28, corridor_width=1, orientation=0. Bounds: branches 1–64, depth 1–126, goal 2–126, width 1–4, clockwise quarter turns 0–3. Dimensions are 8–128; odd rotations require a square. Geometry clips branches/goal to grid bounds. UI weakness experiments explicitly use 32×32 and the current seed. Branch parameters apply to DFS; seed/orientation apply to all families. Rotations deliberately can change the effect of fixed neighbor ordering.

| Family | Construction / objective |
| --- | --- |
| DFS deep trap | Top corridor with seeded downward dead ends. LIFO N/E/S/W pushes select south before east on the default orientation. |
| Greedy barrier & terrain trap | Long barrier, offset entrance requiring movement away from the nearby goal, and a heavy terrain toll on the tempting route. It exposes h-only ordering's cost blindness. The reported cost effect includes terrain; it is not a claim of uniform-terrain Greedy failure. |
| A* low information | Serpentine corridors and seeded misleading openings. Manhattan distance cannot represent long detours; actual expansion similarity is measured. |
| BFS weighted lesson | 29-edge high-cost route versus a 33-edge normal-cost route. BFS retains its step-count objective; Dijkstra optimizes weighted cost. |

Thresholds are public in [`config/stress-thresholds.json`](../config/stress-thresholds.json), version `stress-effect/1`, consumed by Rust and shown in Analysis. Strong/moderate thresholds respectively: DFS expanded ratio ≥2 / ≥1.25, Greedy cost ratio ≥1.25 / ≥1.01, A* expansion ratio ≥0.9 / ≥0.7, BFS cost ratio ≥1.5 / ≥1.01. A missing denominator/reference has no badge or invented ratio. Corpus hit criteria are DFS ≥1.25, Greedy cost ratio >1, A* ≥0.9, BFS cost ratio >1; hit counts are distinct from badge counts.

## STRESS CASE RESULTS

Seeds 0–99 for every family, defaults above. All rows including misses are retained in [`phase11-stress-gate.json`](../phase11-stress-gate.json).

| Stress scenario | Effect hits / seeds | Measured ratio min / mean / max |
| --- | ---: | ---: |
| DFS expanded / best other contender expanded | 100 / 100 | 7.143 / 7.416 / 7.786 |
| Greedy cost / Dijkstra optimal cost | 100 / 100 | 1.119 / 2.704 / 4.478 |
| A* expanded / Dijkstra expanded | 69 / 100 | 0.711 / 0.924 / 0.994 |
| BFS cost / Dijkstra optimal cost | 100 / 100 | 5.636 / 6.406 / 7.606 |

Greedy's mean expanded / Dijkstra expanded is 0.0739: fewer expansions do not establish better path cost. BFS's step count is 29 versus Dijkstra's 33 in this corpus, with the step inequality verified for every seed. A*'s 31 below-threshold cases are listed explicitly in [`phase11-summary.json`](../phase11-summary.json); they are not hidden or discarded.

## DETERMINISM / REPLAY

- 400 repeated maps, 4,000 executions, 2,000 canonical comparisons and 164,172 seek/prefix checks: **0 map mismatch, 0 event/hash mismatch, 0 replay divergence**.
- Phase 0 regression: 600 runs/replays, 16,950 seeks, zero errors.
- Phase 1 regression: 1,000 runs/replays, 24,966 seeks, zero errors.
- All 360 original benchmark hashes and event counts match Phase 1 exactly. A legacy saved JSON fixture still validates and reproduces its old canonical hash.
- Imported analysis is reconstructed from saved logs; import never invokes a new search. Laboratory exports retain all five actual logs and their heuristic configurations.

## Heuristic / observation definitions

All four integer heuristics are admissible and consistent for this **four-direction movement model with destination costs ≥1**. This does not grant Greedy an optimality guarantee.

- Average expanded h = sum of the configured heuristic at actually expanded cells / expanded count.
- Initial h = h(start).
- Effective branching **proxy** = expanded^(1 / solution edge count). This is a named proxy, not the branching factor obtained by solving a geometric-series equation; undefined for zero/no solution length.
- Expansion ratio = expanded / measured Dijkstra expanded. Missing Dijkstra remains undefined.
- Guidance / expansion gain = 1 − expansion ratio. A negative value is retained.
- Cost optimality compares against an available Dijkstra result, or the guaranteed admissible A* result when Dijkstra is absent. The reference algorithm is recorded explicitly. Unreachable paths remain undefined.
- Step optimality uses an available BFS reference; otherwise undefined.
- Cost overhead = path cost / optimal cost − 1; zero-cost denominators remain undefined.
- Interpretation strings are deterministic versioned rules. They do not masquerade as measured facts, nor rank a single combined score.

## SEEK BEFORE / AFTER

Both strategies used exactly the original 128×128 corpus: 8 families × seeds 123/456/789 × 5 algorithms = 120 logs. Each strategy used the same 100 seeded targets per log (12,000 samples each). The log event counts and hashes are identical across strategies. Raw samples and per-run results are in [`phase11-seek-profile.json`](../phase11-seek-profile.json).

| Measurement | Original fixed32 | Candidate density128 |
| --- | ---: | ---: |
| Pooled seek p50 ms | 0.833 | 0.833 |
| Pooled seek p95 ms | 2.359 | 2.615 |
| Worst individual-run seek p95 ms | 27.034 | 37.396 |
| Mean checkpoint memory proxy MiB | 36.95 | 121.29 |
| Maximum checkpoint memory proxy MiB | 44 | 176 |
| Mean suffix event count | 1,326.52 | 367.45 |
| Maximum log event count | 228,351 | 228,351 |
| Reconstruction/checkpoint-build p95 ms | 109.164 | 264.908 |
| Serialization p95 ms (same logs) | 93.689 | 93.689 |

Intervals are max(256, ceil(events/32)) and max(256, ceil(events/128)). The memory proxy is snapshot count × node count × Rust `size_of(NodeState)`; it excludes allocator, paths and event logs and is not browser RAM. More snapshots reduce suffix work but increase clone/build/memory cost. **The original strategy is retained**: the candidate did not improve measured p95/long tail and raises mean memory ~3.28×. Both strategies pass replay equality.

The historic Phase 1 maximum seek p95 was 252.207 ms under a different non-isolated host workload. It is not the controlled before measurement and the difference must not be credited as an optimization. Current timings also remain host-dependent.

## BENCHMARK

[`phase11-benchmark.json`](../phase11-benchmark.json) repeats the unchanged 360-record corpus with 50 seek samples per log. [`stress-benchmark.json`](../stress-benchmark.json) adds 180 records: four stress families × three sizes × three seeds × five algorithms, with explicit parameters and generator version. The original [`benchmark.json`](../benchmark.json) remains historical evidence.

| Grid | Compute p50 / p95 ms | Worst run seek p95 ms |
| --- | ---: | ---: |
| 32×32 | 0.398 / 1.362 | 0.246 |
| 64×64 | 1.479 / 6.684 | 0.525 |
| 128×128 | 6.095 / 32.998 | 36.402 |

Compute includes search/event collection and excludes generation, hashing and playback. Benchmark viewer quantiles describe loaded samples; path-cost averages use finite reached-path values. The heatmap is expanded-node count, not a synthetic score or cross-machine wall-clock ranking.

## TESTS / responsive QA

39 Rust tests (23 foundation, 2 Phase 1, 10 Phase 1.1, 4 CLI) and 42 frontend tests passed, alongside fmt, clippy with warnings denied, lint, typecheck and production build. 24 E2E cases cover baseline interactions plus stress selection, ghost/diff, layers, synchronization, markers, history, laboratory export/import, demo hash repeat and benchmark viewer. A real 128×128 five-log heuristic laboratory also restores its final states after normalized scrubbing. Viewports 1920×1080, 1536×864, 1366×768 and 390×844 have operable controls and no horizontal page overflow/runtime errors.

Browser plugin is absent in this session; QA used the repository's Playwright Chromium workflow. Packaged validation used Playwright connected to the real WebView2 through local CDP. Screenshots are in `docs/screenshots/phase11-*`.

## WINDOWS

Actual portable **0.1.1** application at `http://tauri.localhost/`, with Tauri context present and a fresh WebView2 profile. Five Showcase cases, EXPOSE WEAKNESS, Ghost Diff, replay round trips, Inspector 2.0, Analyze, heuristic laboratory, markers, benchmark loading and automatic Demo analysis were verified. Demo restarts have identical hashes. Three original packaged smoke cases also pass. No console errors or external HTTPS requests were observed.

[`windows-phase11-verification.json`](../windows-phase11-verification.json) retains native results and hashes. The portable copy's SHA-256 matches the built executable: `BFFC0558EF84190EC601A9BDD664FBB8F58D24108D27646F7897B2CA07E5C40F`. `output/windows/artifact-manifest.json` contains EXE/ZIP/NSIS checksums. Distribution files are intentionally ignored by Git.

## GITHUB

Same private [`oliverchenOVO/project-arena`](https://github.com/oliverchenOVO/project-arena), branch `main`, with feature-group commits. [`ARENA verification`](https://github.com/oliverchenOVO/project-arena/actions/workflows/ci.yml) checks Rust/frontend/Windows; [`Phase 1.1 stress gate`](https://github.com/oliverchenOVO/project-arena/actions/workflows/phase11.yml) verifies all four stress families across 100 seeds. The final delivery records the observed final-head workflow outcomes.

## KNOWN ISSUES

- Full-state checkpoint cloning remains the dominant seek/memory tradeoff. Density128 was measured and rejected, not promoted as a successful optimization. No guaranteed frame rate or isolated-host performance claim.
- Stress effects depend on layout, seed, parameters and orientation. The Greedy case deliberately combines barrier geometry and terrain cost. A*'s 31 effect misses remain in the report.
- Desktop is primary; narrow controls wrap and the page scrolls. No alternate browser-engine matrix was validated.
- Live LLM provider, signing and clean-machine installer install/uninstall are optional and unimplemented. Portable use requires WebView2 installed; the installer can bootstrap it online.
- Frontend seeds use safe JavaScript integers; the CLI's full u64 domain remains outside the frontend import contract.

## Reproduce

```powershell
cargo test --workspace --release
cargo run --release -p arena-cli -- gate --output phase0-gate.json
cargo run --release -p arena-cli -- gate-phase1 --output phase1-core-gate.json
cargo run --release -p arena-cli -- gate-phase11 --output phase11-stress-gate.json
cargo run --release -p arena-cli -- profile-seek --output phase11-seek-profile.json
cargo run --release -p arena-cli -- benchmark --output phase11-benchmark.json
cargo run --release -p arena-cli -- benchmark-stress --output stress-benchmark.json
node scripts/phase11-report.mjs
npm test
npm run test:e2e
npm run desktop:build
./scripts/package-windows.ps1
```

For packaged smoke, launch the copied EXE with a fresh `WEBVIEW2_USER_DATA_FOLDER` and `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9227`, set `ARENA_CDP_URL=http://127.0.0.1:9227`, then run both `scripts/desktop-smoke.mjs` and `scripts/desktop-phase11-smoke.mjs`.
