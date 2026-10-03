# Phase 1 acceptance — 2026-10-04 (Asia/Taipei)

Local acceptance: **PHASE 1 PASS**. Remote CI status must be read from the linked workflow; local PASS does not imply that an in-progress CI run has completed.

## Implemented

Rust deterministic engine, five algorithms, eight map families, canonical versioned events/hashes, metrics and state footprint proxy, independent correctness oracle, checkpoint replay, headless CLI, JSON benchmark, React/Vite Canvas UI, BUILD/RACE/INSPECT/ANALYZE, 1v1/four contenders, editor validity, shared timeline and all requested speeds, node inspection, deterministic feature analysis, optional explanation provider abstraction, export/import and Tauri Windows packaging.

No synthetic execution/metrics or algorithm reexecution in replay. UI algorithms are executed in Rust in both desktop and browser development adapters. There is no live LLM provider.

## Checks

| Check | Actual evidence |
| --- | --- |
| Rust tests | 29 passed: 4 CLI, 23 foundation, 2 Phase 1/service |
| Rust formatting / Clippy | Root workspace passed; Tauri formatting passed |
| Frontend unit cases | 30 passed, including reduction of a real Rust log and undefined optimal-cost handling |
| Frontend lint / typecheck / production build | Passed |
| Browser E2E | Final run: 18 passed in 1.3 minutes |
| Browser identity | Correct URL/title, nonblank, no framework overlay |
| Browser health | 0 console errors; no horizontal page overflow at 1536px/390px |
| Packaged desktop | Three pairings passed on a real WebView2/Tauri window |
| Portable clean-start | Copied executable launched with a fresh profile; three pairings passed |

The browser plugin entry point timed out; regular Playwright Chromium was used. Browser development URL: `http://127.0.0.1:1431`; desktop URL: `http://tauri.localhost/`. Desktop screenshots were captured from the packaged app through WebView2's local CDP endpoint.

Interaction coverage: BUILD paint/clear and invalid dimensions; BFS vs DFS, Dijkstra vs A*, A* vs Greedy across open, backtracker maze, weighted, corridor and obstacle maps; end/beginning/event/expansion/backward stepping and speed controls; node inspector; measured analysis; four contenders; export/import without a new search request; missing AI provider; desktop/narrow layouts.

An additional local run initially hit the dev adapter's 5-second wait during compute, then passed after assertions were given a 15-second completion window. No test retries hide this failure. Vite now excludes Rust target directories from watching. The first remote CI failed Tauri formatting; formatted sources are committed for the subsequent run.

## Determinism and replay

| Gate | Seeds | Algorithms | Repeats | Executions/replays | Seek cases | Mismatch/divergence |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Phase 0 | 100 | 3 | 2 | 600 | 16,950 | 0 |
| Phase 1 core | 100 | 5 | 2 | 1,000 | 24,966 | 0 |

Both gates had zero invalid paths/crashes. The five-algorithm gate checked 600 optimality cases for BFS's step objective and Dijkstra/A*'s cost objective. Unit tests additionally cover all four admissible A* heuristics, zero heuristic, tiny/blocked/high-cost/large maps, queue ties, event serialization and invalid imports. Seek checks include checkpoint -1/at/+1, seeded random targets and backward/forward restoration against independently accumulated event prefixes. Execution-owned cost/parent/closed/discovered/frontier arrays match the replay terminal state.

Raw evidence: [`phase0-gate.json`](../phase0-gate.json), [`phase1-core-gate.json`](../phase1-core-gate.json).

## Benchmark

8 families × 3 seeds × 3 dimensions × 5 algorithms = 360 runs. Each includes 50 seek latency samples (18,000 timings); performance timing samples are distinct from correctness seek cases.

| Size | Compute p50 ms | Compute p95 ms | Highest individual-run seek p95 ms |
| --- | ---: | ---: | ---: |
| 32×32 | 0.637 | 3.898 | 0.398 |
| 64×64 | 2.494 | 39.292 | 1.672 |
| 128×128 | 8.677 | 277.989 | 252.207 |

Windows x64, release build; non-isolated host. Compute includes event collection/accounting, excludes generation/hash/serialization/playback. Seek is Rust headless checkpoint replay; these figures do not measure a browser frame rate. All per-run compute/event/checkpoint/replay/seek p50/p95 values are retained in [`benchmark.json`](../benchmark.json). These are measurements, not a performance PASS or universal ranking.

## Windows

- Tauri 2 x64 release executable, unsigned NSIS installer, portable ZIP with license notices.
- Local artifacts: `output/windows/portable/Project-ARENA.exe`, `output/windows/Project-ARENA-0.1.0-portable.zip`, `output/windows/Project ARENA_0.1.0_x64-setup.exe`.
- Portable and original build contain the same compiled executable; the portable copy was actually executed with a new WebView2 data folder.
- HTTPS browser requests were blocked during packaged tests; no remote request occurred. Rust generation/execution and saved-log seek, INSPECT and ANALYZE all worked.
- Portable requires an installed WebView2 Runtime. The installer can bootstrap it online. A clean-machine installer install/uninstall cycle and code signing are not claimed.
- [`windows-launch-verification.json`](../windows-launch-verification.json) records actual terminal metrics, replay equality, node inspector and analysis checks. `output/windows/artifact-manifest.json` records local SHA-256 checksums.

Reproduce the WebView smoke checks by starting the app with `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9223`, optionally assigning a new `WEBVIEW2_USER_DATA_FOLDER`, then running `node scripts/desktop-smoke.mjs`. The profile override is documented in [Microsoft's WebView2 reference](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/webview2-idl?view=webview2-1.0.3967.48); the app command boundary follows [Tauri's Rust command API](https://v2.tauri.app/develop/calling-rust/).

## GitHub

Private repository: [oliverchenOVO/project-arena](https://github.com/oliverchenOVO/project-arena), branch `main`. Repository privacy was verified through CLI and connector metadata. Milestones were committed and pushed individually. The initial CLI token lacked workflow scope; the existing GitHub connector successfully installed the workflow, so that blocker was resolved.

Workflow: [ARENA verification](https://github.com/oliverchenOVO/project-arena/actions/workflows/ci.yml). Core, frontend and Windows build are separate jobs. See the final handoff for the observed run/commit status; this file cannot embed its own future commit hash.

## Known limitations

- 128×128 large-log checkpoint memory/seek latency need optimization; full-state snapshots are capped at 33, trading memory for suffix work.
- No configured AI provider; deterministic analysis remains available.
- No signing, clean-machine installer cycle, Linux/macOS packaging or browser-engine matrix verification.
- Advanced weakness-exposure scenarios, sorting, graph editor and additional algorithm families are deferred.
- Full CLI u64 seeds may exceed JavaScript's safe integer range and are outside the frontend import contract.

Screenshots: [`race`](screenshots/race.png), [`build`](screenshots/build.png), [`inspect`](screenshots/inspect.png), [`analyze`](screenshots/analyze.png), [`packaged Windows`](screenshots/windows-packaged.png).
