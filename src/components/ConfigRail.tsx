import {accents,generators,names} from '../model';
import type {Algorithm,Heuristic,MapConfig} from '../model';
interface Props {map:MapConfig;setMap:(map:MapConfig)=>void;algorithms:Algorithm[];setAlgorithms:(a:Algorithm[])=>void;heuristic:Heuristic;setHeuristic:(h:Heuristic)=>void;busy:boolean;generate:()=>void;randomize:()=>void}
export default function ConfigRail({map,setMap,algorithms,setAlgorithms,heuristic,setHeuristic,busy,generate,randomize}:Props) {
  return <aside className="config-rail"><section><h2>Problem</h2><fieldset disabled={busy}>
    <label>Map family<select aria-label="Map family" value={map.generator} onChange={e=>setMap({...map,generator:e.target.value as MapConfig['generator'],stress:undefined})}>{Object.entries(generators).map(([id,name])=><option key={id} value={id}>{name}</option>)}</select></label>
    <label className="inline-field">Seed<input aria-label="Seed" type="number" min="0" step="1" value={map.seed} onChange={e=>setMap({...map,seed:Number(e.target.value)})}/></label>
    <div className="dimension-fields"><label>Width<input aria-label="Width" type="number" min="1" max="128" value={map.width} onChange={e=>setMap({...map,width:Number(e.target.value)})}/></label><label>Height<input aria-label="Height" type="number" min="1" max="128" value={map.height} onChange={e=>setMap({...map,height:Number(e.target.value)})}/></label></div>
    <label className="inline-field">Walls %<input aria-label="Walls percent" type="number" min="0" max="100" value={map.wall_percent} onChange={e=>setMap({...map,wall_percent:Number(e.target.value)})}/></label>
    <button className="wide" onClick={generate}>↻ Regenerate</button><button className="wide subtle" onClick={randomize}>Next seed</button>
  </fieldset></section><section><h2>Contenders</h2><fieldset disabled={busy}>
    <label>Race format<select aria-label="Race format" value={algorithms.length} onChange={e=>setAlgorithms(e.target.value==='4'?['bfs','dijkstra','astar','greedy']:['dijkstra','astar'])}><option value="2">1 vs 1</option><option value="4">4-way race</option><option value="5" disabled>Heuristic lab + reference</option></select></label>
    {algorithms.map((a,index)=><label key={index} style={{borderLeft:`2px solid ${accents[a]}`,paddingLeft:8}}>Algorithm {String.fromCharCode(65+index)}<select aria-label={`Algorithm ${String.fromCharCode(65+index)}`} disabled={algorithms.length===5} value={a} onChange={e=>setAlgorithms(algorithms.map((old,i)=>i===index?e.target.value as Algorithm:old))}>{Object.entries(names).map(([id,name])=><option key={id} value={id}>{name}</option>)}</select></label>)}
    <label>Heuristic (A* / Greedy)<select aria-label="Heuristic" disabled={algorithms.length===5} value={heuristic} onChange={e=>setHeuristic(e.target.value as Heuristic)}><option value="manhattan">Manhattan</option><option value="euclidean">Euclidean (integer floor)</option><option value="chebyshev">Chebyshev</option><option value="zero">Zero heuristic</option></select></label>
    <p className="hint">4-neighbor: N → E → S → W<br/>Admissible A* heuristics.</p>
  </fieldset></section></aside>;
}
