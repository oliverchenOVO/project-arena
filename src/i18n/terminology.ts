import type {Algorithm,Heuristic} from '../model';
import type {MessageKey} from './index';
// Presentation semantics only. These never enter execution or export data.
export const algorithmDescriptions:Record<Algorithm,string>={bfs:'廣度優先搜尋',dfs:'深度優先搜尋',dijkstra:'最短路徑搜尋',astar:'A* 搜尋',greedy:'貪婪最佳優先搜尋',bidirectional_bfs:'雙向廣度優先搜尋',weighted_astar:'加權啟發式搜尋'};
export const heuristicNames:Record<Heuristic,MessageKey>={zero:'Zero heuristic',manhattan:'Manhattan',euclidean:'Euclidean (integer floor)',chebyshev:'Chebyshev'};
