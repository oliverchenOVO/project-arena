export type Terrain='normal'|'wall'|'mud'|'water'|'heavy';
export type Algorithm='bfs'|'dijkstra'|'astar'|'dfs'|'greedy'|'bidirectional_bfs'|'weighted_astar';
export type Heuristic='manhattan'|'euclidean'|'chebyshev'|'zero';
export type Generator='open'|'obstacles'|'backtracker'|'division'|'rooms'|'corridors'|'weighted'|'heuristic_trap'|'dfs_deep_trap'|'greedy_trap'|'low_information'|'bfs_weighted_lesson';
export interface Grid {width:number;height:number;cells:Terrain[];start:number;goal:number}
export interface StressParams {version:'stress/1';branch_count:number;branch_depth:number;goal_placement:number;corridor_width:number;orientation:number}
export interface MapConfig {seed:number;width:number;height:number;generator:Generator;wall_percent:number;stress?:StressParams}
export interface RunConfig {algorithm:Algorithm;algorithm_version:string;heuristic:Heuristic;neighbor_order:string;tie_break:string;weight_percent?:number}
export type EventKind=
  | {type:'run_started'}
  | {type:'node_discovered'|'node_expanded'|'frontier_added'|'frontier_removed'|'goal_reached';node:number}
  | {type:'edge_evaluated';from:number;to:number;cost:number}
  | {type:'relaxation_compared';node:number;candidate:number}
  | {type:'cost_updated';node:number;g:number;h:number;f:number}
  | {type:'parent_updated';node:number;parent:number}
  | {type:'path_reconstructed';path:number[];cost:number}
  | {type:'search_side'|'direction_expanded';node:number;side:string}
  | {type:'meeting_point';node:number}
  | {type:'weighted_priority';node:number;scaled_f:number;scale:number}
  | {type:'run_completed';found:boolean};
export type ArenaEvent=EventKind & {sequence:number;tick:number};
export interface Metrics {expanded:number;generated:number;edges_evaluated:number;frontier_pushes:number;frontier_pops:number;cost_updates:number;relaxation_comparisons:number;peak_frontier:number;peak_discovered:number;peak_state_entries:number;path_length:number|null;path_cost:number|null;logical_ops:number}
export interface RunLog {schema_version:number;engine_version:string;run_id:string;generator_version:string;rng_version:string;terrain_costs:number[];map_config:MapConfig;grid:Grid;config:RunConfig;events:ArenaEvent[];metrics:Metrics;runtime:{compute_ns:number;events_generated:number;events_per_second:number};canonical_hash:string}
export interface NodeState {discovered:boolean;expanded:boolean;frontier_entries:number;g:number|null;h:number|null;f:number|null;parent:number|null;expanded_at:number|null;expanded_order:number|null;side?:string;meeting?:boolean}
export interface ReplayState {nodes:NodeState[];current:number|null;path:number[];completed:boolean;found:boolean;metrics:Metrics;frontier:number;finiteCosts:number;parents:number}
export interface Features {obstacle_density:number;open_space_ratio:number;corridor_ratio:number;dead_end_count:number;weighted_terrain_ratio:number;path_tortuosity:number|null}
export interface DerivedObservation {algorithm:Algorithm;heuristic:Heuristic;average_h_at_expansion:number|null;effective_branching_proxy:number|null;expansion_ratio_to_dijkstra:number|null;heuristic_guidance_ratio:number|null;path_cost_optimal:boolean|null;step_optimal:boolean|null;cost_ratio_to_optimal:number|null;expanded_vs_best:number|null;stress_effect:number|null;badge:string|null}
export interface Observations {version:string;measured:unknown[];derived:DerivedObservation[];interpretation:{algorithm:Algorithm;text:string;rule_version:string}[];thresholds:unknown;formulas:Record<string,string>;baseline:unknown}
export interface Marker {label:string;cursor:number;event:number;value?:number}
export interface RaceResult {logs:RunLog[];features:Features[];observations?:Observations;markers?:Marker[][];observationV2?:Record<string,unknown>}
export const names:Record<Algorithm,string>={bfs:'BFS',dijkstra:'Dijkstra',astar:'A*',dfs:'DFS',greedy:'Greedy Best-First',bidirectional_bfs:'Bidirectional BFS',weighted_astar:'Weighted A*'};
export const accents:Record<Algorithm,string>={bfs:'#9c9fed',dijkstra:'#569ee7',astar:'#4dd6c6',dfs:'#d8a969',greedy:'#d18bb0',bidirectional_bfs:'#4dd6c6',weighted_astar:'#ecb777'};
export const generators:Record<Generator,string>={open:'Open field',obstacles:'Random obstacles',backtracker:'Backtracker maze',division:'Division maze',rooms:'Rooms',corridors:'Narrow corridors',weighted:'Weighted terrain',heuristic_trap:'Heuristic stress case',dfs_deep_trap:'DFS deep trap · stress/1',greedy_trap:'Greedy barrier & terrain trap · stress/1',low_information:'A* low information · stress/1',bfs_weighted_lesson:'BFS weighted lesson · stress/1'};
export const defaultMap:MapConfig={seed:123,width:32,height:32,generator:'weighted',wall_percent:15};
export function runConfig(algorithm:Algorithm,heuristic:Heuristic):RunConfig {return {algorithm,heuristic:algorithm==='astar'||algorithm==='greedy'||algorithm==='weighted_astar'?heuristic:'zero',algorithm_version:'1',neighbor_order:'N,E,S,W',tie_break:'priority,h,insertion'};}
export function metric(value:number|null|undefined):string {return value==null?'—':value.toLocaleString('en-US');}
