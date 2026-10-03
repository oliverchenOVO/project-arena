export type Terrain='normal'|'wall'|'mud'|'water'|'heavy';
export type Algorithm='bfs'|'dijkstra'|'astar'|'dfs'|'greedy';
export type Heuristic='manhattan'|'euclidean'|'chebyshev'|'zero';
export type Generator='open'|'obstacles'|'backtracker'|'division'|'rooms'|'corridors'|'weighted'|'heuristic_trap';
export interface Grid {width:number;height:number;cells:Terrain[];start:number;goal:number}
export interface MapConfig {seed:number;width:number;height:number;generator:Generator;wall_percent:number}
export interface RunConfig {algorithm:Algorithm;algorithm_version:string;heuristic:Heuristic;neighbor_order:string;tie_break:string}
export type EventKind=
  | {type:'run_started'}
  | {type:'node_discovered'|'node_expanded'|'frontier_added'|'frontier_removed'|'goal_reached';node:number}
  | {type:'edge_evaluated';from:number;to:number;cost:number}
  | {type:'relaxation_compared';node:number;candidate:number}
  | {type:'cost_updated';node:number;g:number;h:number;f:number}
  | {type:'parent_updated';node:number;parent:number}
  | {type:'path_reconstructed';path:number[];cost:number}
  | {type:'run_completed';found:boolean};
export type ArenaEvent=EventKind & {sequence:number;tick:number};
export interface Metrics {expanded:number;generated:number;edges_evaluated:number;frontier_pushes:number;frontier_pops:number;cost_updates:number;relaxation_comparisons:number;peak_frontier:number;peak_discovered:number;peak_state_entries:number;path_length:number|null;path_cost:number|null;logical_ops:number}
export interface RunLog {schema_version:number;engine_version:string;run_id:string;generator_version:string;rng_version:string;terrain_costs:number[];map_config:MapConfig;grid:Grid;config:RunConfig;events:ArenaEvent[];metrics:Metrics;runtime:{compute_ns:number;events_generated:number;events_per_second:number};canonical_hash:string}
export interface NodeState {discovered:boolean;expanded:boolean;frontier_entries:number;g:number|null;h:number|null;f:number|null;parent:number|null;expanded_at:number|null}
export interface ReplayState {nodes:NodeState[];current:number|null;path:number[];completed:boolean;found:boolean;metrics:Metrics;frontier:number;finiteCosts:number;parents:number}
export interface Features {obstacle_density:number;open_space_ratio:number;corridor_ratio:number;dead_end_count:number;weighted_terrain_ratio:number;path_tortuosity:number|null}
export interface RaceResult {logs:RunLog[];features:Features[]}
export const names:Record<Algorithm,string>={bfs:'BFS',dijkstra:'Dijkstra',astar:'A*',dfs:'DFS',greedy:'Greedy best-first'};
export const accents:Record<Algorithm,string>={bfs:'#9c9fed',dijkstra:'#569ee7',astar:'#4dd6c6',dfs:'#d8a969',greedy:'#d18bb0'};
export const generators:Record<Generator,string>={open:'Open field',obstacles:'Random obstacles',backtracker:'Backtracker maze',division:'Division maze',rooms:'Rooms',corridors:'Narrow corridors',weighted:'Weighted terrain',heuristic_trap:'Heuristic stress case'};
export const defaultMap:MapConfig={seed:123,width:32,height:32,generator:'weighted',wall_percent:15};
export function runConfig(algorithm:Algorithm,heuristic:Heuristic):RunConfig {return {algorithm,heuristic:algorithm==='astar'||algorithm==='greedy'?heuristic:'zero',algorithm_version:'1',neighbor_order:'N,E,S,W',tie_break:'priority,h,insertion'};}
export function metric(value:number|null|undefined):string {return value==null?'—':value.toLocaleString('en-US');}
