import {useSyncExternalStore} from 'react';
import {accents,metric,names} from '../model';
import type {Algorithm,Grid} from '../model';
import type {RacePlayback} from '../playback';
import GridCanvas from './GridCanvas';
export function RaceMetrics({store,index}:{store:RacePlayback;index:number}) {
  useSyncExternalStore(store.subscribe,store.snapshot);
  const state=store.replays[index].state;
  return <div className="race-metrics"><span>Expanded <b>{metric(state.metrics.expanded)}</b></span><span>Frontier <b>{metric(state.frontier)}</b></span><span>Path cost <b>{metric(state.metrics.path_cost)}</b></span></div>;
}
export default function RacePanels({grid,algorithms,store,onCell,selected}:{grid:Grid;algorithms:Algorithm[];store:RacePlayback|null;onCell:(node:number)=>void;selected:number|null}) {
  const actual=store?store.logs.map(log=>log.config.algorithm):algorithms;
  return <div className={`race-panels count-${actual.length}`}>{actual.map((a,index)=><article className="viewport" key={index} style={{'--accent':accents[a]} as React.CSSProperties}><header><h2>{names[a]}</h2><span>Grid {grid.width} × {grid.height}</span></header><GridCanvas grid={grid} accent={accents[a]} store={store} index={index} onCell={onCell} selected={selected}/>{store?<RaceMetrics store={store} index={index}/>:<div className="race-metrics"><span>Expanded <b>—</b></span><span>Frontier <b>—</b></span><span>Path cost <b>—</b></span></div>}</article>)}</div>;
}
