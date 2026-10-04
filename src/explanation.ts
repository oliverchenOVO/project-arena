import type {Features,RunLog} from './model';
export interface StructuredSummary {measured:unknown[];derived:unknown[];hypotheses:string[]}
export interface ObservationPayload {problem_metadata:unknown;measured:unknown[];derived:unknown[];interpretation:unknown[]}
export interface ExplanationProvider {id:string;explain(summary:ObservationPayload):Promise<string>}
/** Optional adapter; never called by execution, replay or benchmarking. */
export class UnavailableProvider implements ExplanationProvider {
  id='unavailable';
  async explain():Promise<string>{throw new Error('AI explanation unavailable: no provider configured');}
}
export function summarize(logs:RunLog[],features:Features[]):StructuredSummary {
  const baseline=logs.find(log=>log.config.algorithm==='dijkstra');
  return {
    measured:logs.map(log=>({algorithm:log.config.algorithm,metrics:log.metrics,computeNs:log.runtime.compute_ns,optimalCostMatchesDijkstra:baseline&&baseline.metrics.path_cost!==null?log.metrics.path_cost===baseline.metrics.path_cost:null})),
    derived:logs.map((log,index)=>({algorithm:log.config.algorithm,...features[index],expansionRatioToDijkstra:baseline&&baseline.metrics.expanded?log.metrics.expanded/baseline.metrics.expanded:null})),
    hypotheses:logs.some(log=>log.config.algorithm==='astar')?['A goal-directed admissible heuristic may reduce expansion when map geometry provides useful distance information. This explanation is a hypothesis; the table above contains the measured counts.']:[],
  };
}
