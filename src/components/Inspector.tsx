import {heuristicNames} from '../i18n/terminology';
import {useLocale} from '../i18n';
import {useState,useSyncExternalStore} from 'react';
import {metric,names} from '../model';
import type {Grid} from '../model';
import type {RacePlayback} from '../playback';
import {nodeHistory} from '../laboratory';
function NodeSelector({grid,node,onSelect}:{grid:Grid;node:number|null;onSelect:(node:number)=>void}) {
  const {t}=useLocale();
  const [x,setX]=useState(node===null?'0':String(node%grid.width));
  const [y,setY]=useState(node===null?'0':String(Math.floor(node/grid.width)));
  return <form className="node-selector" onSubmit={event=>{event.preventDefault();const col=Number(x),row=Number(y);if(x!==''&&y!==''&&Number.isInteger(col)&&Number.isInteger(row)&&col>=0&&col<grid.width&&row>=0&&row<grid.height)onSelect(row*grid.width+col);}}>
    <label>{t('Column (x)')}<input aria-label={t('Column (x)')} type="number" min={0} max={grid.width-1} step={1} required value={x} onChange={e=>setX(e.target.value)}/></label>
    <label>{t('Row (y)')}<input aria-label={t('Row (y)')} type="number" min={0} max={grid.height-1} step={1} required value={y} onChange={e=>setY(e.target.value)}/></label>
    <button type="submit">{t('Inspect coordinates')}</button><small>{t('Coordinates start at zero. Selecting a node does not run a search.')}</small>
  </form>;
}
export default function Inspector({grid,store,node,onSelect}:{grid:Grid;store:RacePlayback;node:number|null;onSelect:(node:number)=>void}) {
  const {t}=useLocale();
  useSyncExternalStore(store.subscribe,store.snapshot);
  const selector=<NodeSelector key={node??'none'} grid={grid} node={node} onSelect={onSelect}/>;
  if(node===null)return <section className="inspector"><h2>{t("Node inspector")}</h2>{selector}<p>{t("Select a cell in either viewport to inspect the recorded state.")}</p></section>;
  return <section className="inspector"><h2>{t("Node (")}{node%grid.width}, {Math.floor(node/grid.width)}) · {t(grid.cells[node])}</h2><p className="hint">{t("Cross-algorithm comparison at the current replay position; absent events are omitted.")}</p>{selector}<div className="node-details">{store.replays.map((replay,index)=>{const state=replay.state.nodes[node];const a=replay.log.config.algorithm;const history=nodeHistory(replay.log.events,node,replay.cursor);return <div key={index}><h3>{names[a]} · {t(heuristicNames[replay.log.config.heuristic])}</h3><dl><dt>{t("Status")}</dt><dd>{t(t(state.expanded?'CLOSED':state.frontier_entries?'FRONTIER':state.discovered?'DISCOVERED':'UNVISITED'))}</dd><dt>{t(state.side==="backward"?"Distance from goal":"g / distance")}</dt><dd>{metric(state.g)}</dd>{a==='astar'||a==='greedy'||a==='weighted_astar'?<><dt>{t("h")}</dt><dd>{metric(state.h)}</dd><dt>{t(t(a==='greedy'?'g + h (priority = h)':'f'))}</dt><dd>{metric(state.f)}</dd></>:null}<dt>{t("Parent")}</dt><dd>{state.parent===null?'—':`(${state.parent%grid.width}, ${Math.floor(state.parent/grid.width)})`}</dd><dt>{t("Expanded at")}</dt><dd>{state.expanded_at===null?(t(t(replay.state.completed?'Never expanded':'Not yet expanded'))):`#${state.expanded_at}`}</dd><dt>{t("Parent changed")}</dt><dd>{history.changes}{t("times (initial assignment excluded)")}</dd></dl><details><summary>{t("Event history ·")}{history.history.length}{t("records")}</summary><ol className="event-history">{history.history.map(e=><li key={e.sequence}><button onClick={()=>store.jump(index,e.sequence+1)}>{t(e.type.replaceAll('_',' '))} #{e.sequence}</button></li>)}</ol></details></div>;})}</div></section>;
}
