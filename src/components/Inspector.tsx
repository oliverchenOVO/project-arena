import {heuristicNames} from '../i18n/terminology';
import {useLocale} from '../i18n';
import {useSyncExternalStore} from 'react';
import {metric,names} from '../model';
import type {Grid} from '../model';
import type {RacePlayback} from '../playback';
import {nodeHistory} from '../laboratory';
export default function Inspector({grid,store,node}:{grid:Grid;store:RacePlayback;node:number|null}) {
  const {t}=useLocale();
  useSyncExternalStore(store.subscribe,store.snapshot);
  if(node===null)return <section className="inspector"><h2>{t("Node inspector")}</h2><p>{t("Select a cell in either viewport to inspect the recorded state.")}</p></section>;
  return <section className="inspector"><h2>{t("Node (")}{node%grid.width}, {Math.floor(node/grid.width)}) · {t(grid.cells[node])}</h2><p className="hint">{t("Cross-algorithm comparison at the current replay position; absent events are omitted.")}</p><div className="node-details">{store.replays.map((replay,index)=>{const state=replay.state.nodes[node];const a=replay.log.config.algorithm;const history=nodeHistory(replay.log.events,node,replay.cursor);return <div key={index}><h3>{names[a]} · {t(heuristicNames[replay.log.config.heuristic])}</h3><dl><dt>{t("Status")}</dt><dd>{t(t(state.expanded?'CLOSED':state.frontier_entries?'FRONTIER':state.discovered?'DISCOVERED':'UNVISITED'))}</dd><dt>{t("g / distance")}</dt><dd>{metric(state.g)}</dd>{a==='astar'||a==='greedy'?<><dt>{t("h")}</dt><dd>{metric(state.h)}</dd><dt>{t(t(a==='greedy'?'g + h (priority = h)':'f'))}</dt><dd>{metric(state.f)}</dd></>:null}<dt>{t("Parent")}</dt><dd>{state.parent===null?'—':`(${state.parent%grid.width}, ${Math.floor(state.parent/grid.width)})`}</dd><dt>{t("Expanded at")}</dt><dd>{state.expanded_at===null?(t(t(replay.state.completed?'Never expanded':'Not yet expanded'))):`#${state.expanded_at}`}</dd><dt>{t("Parent changed")}</dt><dd>{history.changes}{t("times (initial assignment excluded)")}</dd></dl><details><summary>{t("Event history ·")}{history.history.length}{t("records")}</summary><ol className="event-history">{history.history.map(e=><li key={e.sequence}><button onClick={()=>store.jump(index,e.sequence+1)}>{t(e.type.replaceAll('_',' '))} #{e.sequence}</button></li>)}</ol></details></div>;})}</div></section>;
}
