# Phase 1.2 — Localization & Presentation

Version 0.1.2 continues the existing repository. Engine/schema/algorithm versions, RNG, search ordering, tie-breaking, event semantics, canonical serialization and checkpoint strategy are unchanged. No search algorithm was added.

## Localization

The central `src/i18n` dictionaries have **374 identical keys** for `zh-TW` and `en`, with **zero missing keys**. Tests check identical nonempty keysets, every literal component translation call, fallback behavior, language precedence and interpolation. Presentation names are separate from canonical algorithm, generator and event IDs.

Locale precedence is saved `arena.locale.v1`, browser language list, then Traditional Chinese. The locale context wraps the application without a locale key or remount. Its effect changes document language and preference storage only. The existing algorithm execution and playback effects do not depend on locale. A live-switch E2E pauses a real run, changes language, checks the exact cursor and metric values, and compares exported JSON byte for byte. It also imports the same saved data in both languages and counts zero additional race requests.

CJK text uses Segoe UI / Microsoft JhengHei / PingFang TC / system fonts. Hashes, numeric inputs and metric values retain monospace presentation. The ARENA wordmark and short CS algorithm names stay recognizable in both languages. Chinese descriptions explain BFS, DFS, Dijkstra, A* and Greedy Best-First. Rust observation sentences are translated at the presentation boundary; the original rule/version and structured JSON remain unchanged.

## Showcase and presentation

Five landing cards execute the existing fixed map/seed/config presets: Open Navigation, Deep Maze, Greedy Trap, Weighted-Cost Lesson and Weak Heuristic Maze. All five are exercised in both locales by browser E2E and the actual Windows executable.

The guided Open A*/Dijkstra demo executes Rust, then pauses on three actual event markers. Callout counts come from the selected recorded replay prefix. Continuing playback reaches Analysis, which shows three educational conclusions. Restart produces identical canonical hashes. “60-second” is an introductory format; exact duration depends on speed and user pauses.

The settings rail collapses after execution and saves its preference. First entry has a dismissible three-step guide; “Do not show again” persists. Presentation Mode reduces secondary content and supports Escape. Completed-race details remain expandable, allowing the principal playback controls to stay in the first viewport at 1536×864 and 1920×1080.

Analysis derives its headline from completed measured logs. Equal reachable A*/Dijkstra costs plus fewer A* expansions produce the calculated reduction percentage; other comparisons use a neutral headline. Expanded nodes, logical operations, peak frontier and path cost have independent horizontal charts and units. Detailed tables remain available. MEASURED, DERIVED and INTERPRETATION have explanatory tooltips; inference is not presented as another measured fact. The unconfigured AI provider is a muted informational panel. Benchmark remains a saved-data viewer; compute time excludes playback.

## Accessibility and responsive QA

Visible controls and accessible labels are localized, including navigation, inputs, replay transports, timeline, markers, node histories and import/export. Tests use roles and localized semantic labels rather than assuming English UI. Keyboard navigation and Escape work with the existing visible focus indicators. This is targeted interaction/accessibility QA, not a claim of full WCAG certification or screen-reader access to every canvas cell.

Both locales exercise Landing, settings, Race, timeline/markers, Inspector and Analysis at **1920×1080, 1536×864, 1366×768 and 390×844**. Tests check settled page width without horizontal overflow and verify primary playback controls remain visible in the two large desktop viewports. Dense analysis/benchmark tables have an internal scroll container. Chinese cards and controls wrap without page overflow; long native generator labels are shortened for readability.

Browser plugin was not available, so validation used the existing Playwright configuration against `http://127.0.0.1:1431/`. Page identity, meaningful content, absence of framework error overlays, console health, interaction results and screenshot evidence were checked. Native smoke uses Playwright CDP against the real Tauri WebView at `http://tauri.localhost/`, not the Vite adapter.

## Gates and tests

| Check | Result |
| --- | --- |
| Rust fmt, Tauri fmt, clippy | PASS |
| Rust release tests | 39 PASS |
| Frontend typecheck, lint, production build | PASS |
| Frontend unit tests | 48 PASS |
| Full browser E2E | 33 PASS, zero failed/flaky/skipped |
| Existing 24 flows | All execute with zh-TW UI |
| Added localization flows | 3 zh-TW, 3 en, 3 switch/setup/guided-demo cases |
| Phase 0 | 600 executions/replays, 16,950 seek checks |
| Phase 1 | 1,000 executions/replays, 24,966 seek checks |
| Phase 1.1 stress gate | 400 maps, 4,000 repeated executions, 164,172 seek checks |
| Repeated determinism | 2,800 hash comparisons, zero mismatch |
| Historical hashes | 2,540 comparisons, zero mismatch |
| Historical benchmark metrics/events | 360 ordinary + 180 stress records unchanged |
| Replay divergences | 0 across all gates |
| Packaged Windows smoke | 12 bilingual cases PASS; zero console errors/external requests |

The 2,540 historical comparisons comprise 360 original benchmark hashes, 180 stress benchmark hashes and 400×5 Phase 1.1 stress hashes. Runtime measurements are deliberately excluded; new host timings may differ. Every compared event count and algorithmic metric remains identical. The large-log checkpoint tradeoff and prior controlled seek profile remain documented in [Phase 1.1](phase11.md); this phase does not claim a replay performance optimization.

Machine-readable evidence: [summary](../phase12-summary.json), [Phase 0](../phase12-phase0-gate.json), [Phase 1](../phase12-phase1-gate.json), [stress gate](../phase12-stress-gate.json), [ordinary benchmark](../phase12-benchmark.json), [stress benchmark](../phase12-stress-benchmark.json), [native verification](../windows-phase12-verification.json).

## Windows and screenshots

The x64 release executable, portable ZIP and unsigned NSIS installer are version 0.1.2. Packaging verifies that the portable copy matches the built binary and records byte sizes and SHA-256 hashes. The new executable is launched with a fresh WebView profile. Both languages exercise five fixed scenarios, replay, inspection, independent charts, stress generation, four-heuristic reference, saved-log import and benchmark loading. The native report includes the actual artifact manifest and hashes.

Ten formal screenshots are from the packaged Tauri executable without browser chrome, devtools, extensions or banners:

| View | Screenshot |
| --- | --- |
| Chinese Landing | [Landing](screenshots/phase12-zh-landing.png) |
| Chinese A*/Dijkstra Race | [Race](screenshots/phase12-zh-astar-dijkstra-race.png) |
| Chinese Overlay Diff | [Diff](screenshots/phase12-zh-overlay-diff.png) |
| Chinese Stress Lab | [Stress](screenshots/phase12-zh-stress-lab.png) |
| Chinese A*/Dijkstra Inspector | [Inspector 1](screenshots/phase12-zh-inspector-astar-dijkstra.png) |
| Chinese BFS/Dijkstra Inspector | [Inspector 2](screenshots/phase12-zh-inspector-bfs-dijkstra.png) |
| Chinese Analysis | [Analysis](screenshots/phase12-zh-analysis.png) |
| Chinese Benchmark | [Benchmark](screenshots/phase12-zh-benchmark.png) |
| English Race | [English](screenshots/phase12-en-race.png) |
| Windows Presentation Mode | [Packaged Windows](screenshots/phase12-windows-packaged.png) |

Four additional Chinese settings screenshots document the required browser viewport sizes. English README remains the main entry, with [README.zh-TW.md](../README.zh-TW.md) covering overview, features, algorithms, determinism, screenshots, development, Windows, benchmark and limitations.

## GitHub and limitations

Repository remains private at [oliverchenOVO/project-arena](https://github.com/oliverchenOVO/project-arena), branch `main`. The existing verification and stress workflows exercise the changed tests and Windows build. Consult [Actions](https://github.com/oliverchenOVO/project-arena/actions) for the final commit's result; local evidence does not substitute for that remote result.

No optional AI provider is configured. Signing and clean-machine installer tests are not claimed. Guided demo duration varies with pauses. Canvas cell inspection remains a pointer-driven interface; localized labels and keyboard-operated surrounding controls do not imply a complete accessible grid representation.
