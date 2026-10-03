# Execution contracts (schema 1)

- Movement: four orthogonal directions, North → East → South → West.
- Cost: pay destination terrain; start costs zero. Normal 1, mud 3, water 5, heavy 10; walls impassable.
- BFS optimizes steps, not terrain cost. Dijkstra and admissible A* optimize cost.
- DFS uses LIFO discovery-once search; Greedy prioritizes h and discovers each node once. Neither guarantees shortest paths. DFS neighbor events retain N/E/S/W order, so the stack explores the last eligible pushed node first.
- Heap ordering: lower priority, lower heuristic, earlier insertion sequence. Lazy stale entries are counted as real queue pops but never expansions.
- RNG: SplitMix64 with explicit wrapping arithmetic and unbiased bounded sampling; seed is an unsigned 64-bit integer.
- Heuristics use integer values: Manhattan, floor Euclidean, Chebyshev, zero. All are admissible and consistent for four-direction movement and minimum terrain cost 1.
- Canonical identity: SHA-256 of compact serde JSON of schema version, engine version, problem, configuration and events. No maps with unordered keys or floats; measured runtime is excluded. Determinism is promised within the recorded engine/schema versions.
- Tick: expansion ordinal; sequence: zero-based event ordinal. No wall clock drives execution.
- Events are tagged variants with only relevant fields. Run ID is a deterministic input hash, present once in the log; algorithm and immutable metadata are also log-level fields, avoiding repetition in every event.
- Logical operations: each expansion, edge evaluation, frontier push, frontier pop, and relaxation comparison is +1. Discovery, parent assignment and visualization notifications are not operations. A relaxation comparison is counted for every evaluated traversable edge, including rejected candidates.
- Generated means first discovery of a node (including start). Path length is number of edges. Unreachable runs have null path length/cost, not zero.
- Footprint proxy: peak queue entries (including stale entries), discovered nodes, and combined active state entries (discovered + finite cost entries + parent entries + queue entries). This is not RAM or bytes.
- Runtime measures search plus event collection; JSON serialization, hashing, map generation and playback are outside compute duration. There is no timing-only no-events kernel in Phase 0.
- Replay consumes saved events only. Checkpoints use a minimum interval of 256 events, increased to bound full-state snapshots to at most 33. Seek applies only the suffix after its nearest checkpoint.
- Random obstacle maps may be disconnected. Generators do not silently guarantee reachability or repair walls, beyond making start/goal traversable.
