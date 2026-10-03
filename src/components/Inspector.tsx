import {useSyncExternalStore} from 'react';
import {metric,names} from '../model';
import type {Grid} from '../model';
import type {RacePlayback} from '../playback';
export default function Inspector({grid,store,node}:{grid:Grid;store:RacePlayback;node:number|null}) {
  useSyncExternalStore(store.subscribe,store.snapshot);
  if(node===null)return <section className="inspector"><h2>Node inspector</h2><p>Select a cell in either viewport to inspect the recorded state.</p></section>;
  return <section className="inspector"><h2>Node ({node%grid.width}, {Math.floor(node/grid.width)}) · {grid.cells[node]}</h2><div className="node-details">{store.replays.map((replay,index)=>{const state=replay.state.nodes[node];const a=replay.log.config.algorithm;return <div key={index}><h3>{names[a]}</h3><dl><dt>Status</dt><dd>{state.expanded?'CLOSED':state.frontier_entries?'FRONTIER':state.discovered?'DISCOVERED':'UNVISITED'}</dd><dt>g / distance</dt><dd>{metric(state.g)}</dd>{a==='astar'||a==='greedy'?<><dt>h</dt><dd>{metric(state.h)}</dd><dt>{a==='greedy'?'g + h (priority = h)':'f'}</dt><dd>{metric(state.f)}</dd></>:null}<dt>Parent</dt><dd>{state.parent===null?'—':`(${state.parent%grid.width}, ${Math.floor(state.parent/grid.width)})`}</dd><dt>Expanded at</dt><dd>{state.expanded_at===null?'—':`#${state.expanded_at}`}</dd></dl></div>;})}</div></section>;
}
