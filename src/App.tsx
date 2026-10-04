import {useCallback,useEffect,useRef,useState} from 'react';
import {backend,exportRace,importRace} from './backend';
import {defaultMap,runConfig} from './model';
import type {Algorithm,Grid,Heuristic,MapConfig,RaceResult} from './model';
import {clearGrid,editCell,validateMap} from './editor';
import type {Tool} from './editor';
import {RacePlayback} from './playback';
import ConfigRail from './components/ConfigRail';
import RacePanels from './components/RacePanels';
import GridCanvas from './components/GridCanvas';
import Timeline from './components/Timeline';
import Inspector from './components/Inspector';
import Analysis from './components/Analysis';
import LabControls from './components/LabControls';
import Benchmark from './components/Benchmark';
import {presets} from './laboratory';

type Mode='BUILD'|'RACE'|'INSPECT'|'ANALYZE'|'BENCHMARK';
const tools:Tool[]=['start','goal','wall','mud','water','heavy','erase'];
export default function App() {
  const [mode,setMode]=useState<Mode>('RACE');
  const [map,setMap]=useState<MapConfig>(defaultMap);
  const [appliedMap,setAppliedMap]=useState<MapConfig>(defaultMap);
  const [grid,setGrid]=useState<Grid|null>(null);
  const [algorithms,setAlgorithms]=useState<Algorithm[]>(['dijkstra','astar']);
  const [heuristic,setHeuristic]=useState<Heuristic>('manhattan');
  const [tool,setTool]=useState<Tool>('wall');
  const [selected,setSelected]=useState<number|null>(null);
  const [race,setRace]=useState<RaceResult|null>(null);
  const [store,setStore]=useState<RacePlayback|null>(null);
  const [busy,setBusy]=useState(true);
  const [error,setError]=useState('');
  const input=useRef<HTMLInputElement>(null);
  const [demo,setDemo]=useState(false);
  useEffect(()=>{
    let active=true;
    backend.generate(defaultMap).then(result=>{if(active)setGrid(result.grid);}).catch(error=>{if(active)setError(String(error));}).finally(()=>{if(active)setBusy(false);});
    return ()=>{active=false;};
  },[]);
  useEffect(()=>{
    if(!store)return;
    let frame=0;let previous=performance.now();
    const update=(now:number)=>{store.update(Math.min(100,now-previous));previous=now;frame=requestAnimationFrame(update);};
    frame=requestAnimationFrame(update);const unsubscribe=store.subscribe(()=>{if(demo&&store.cursor===store.length){setMode('ANALYZE');setDemo(false);}});return ()=>{cancelAnimationFrame(frame);unsubscribe();};
  },[store,demo]);
  const resetRace=()=>{store?.pause();setStore(null);setRace(null);setSelected(null);};
  const generate=async(config=map)=>{
    if(busy)return;
    setError('');setBusy(true);
    try {validateMap(config);const result=await backend.generate(config);resetRace();setGrid(result.grid);setAppliedMap({...config});setMap({...config});}
    catch(error){setError(error instanceof Error?error.message:String(error));}
    finally{setBusy(false);}
  };
  const run=async()=>{
    if(busy||!grid)return;
    setError('');setBusy(true);store?.pause();setDemo(false);
    try {
      const result=await backend.race(grid,appliedMap,algorithms.length===5&&race?.logs.length===5?race.logs.map(l=>l.config):algorithms.map(a=>runConfig(a,heuristic)));
      const next=new RacePlayback(result.logs);setRace(result);setStore(next);setMode('RACE');next.toggle();
    }catch(error){setError(error instanceof Error?error.message:String(error));}finally{setBusy(false);}
  };
  const scenario=async(config:MapConfig,pair:Algorithm[],isDemo=false)=>{
    if(busy)return;setBusy(true);setError('');store?.pause();setDemo(false);
    try{validateMap(config);const generated=await backend.generate(config);const result=await backend.race(generated.grid,config,pair.map(a=>runConfig(a,'manhattan')));const next=new RacePlayback(result.logs);next.setSync('percentage');setMap(config);setAppliedMap(config);setGrid(generated.grid);setAlgorithms(pair);setHeuristic('manhattan');setRace(result);setStore(next);setSelected(null);setMode('RACE');setDemo(isDemo);next.toggle();}
    catch(error){setError(String(error));}finally{setBusy(false);}
  };
  const heuristicLab=async(a:'astar'|'greedy')=>{
    if(busy||!grid)return;setBusy(true);setError('');store?.pause();setDemo(false);
    try{const configs=(['zero','manhattan','euclidean','chebyshev'] as Heuristic[]).map(h=>runConfig(a,h));configs.push(runConfig('dijkstra','zero'));const result=await backend.race(grid,appliedMap,configs);const next=new RacePlayback(result.logs);next.setSync('percentage');setRace(result);setStore(next);setSelected(null);setMode('RACE');next.toggle();}
    catch(error){setError(String(error));}finally{setBusy(false);}
  };
  const onCell=useCallback((node:number)=>{
    setSelected(node);
    if(mode==='BUILD'&&!busy){setGrid(old=>old?editCell(old,node,tool):old);setRace(null);setStore(null);}
  },[mode,busy,tool]);
  const load=async(file:File)=>{
    setBusy(true);setError('');store?.pause();setDemo(false);
    try {const result=await importRace(file);const first=result.logs[0];setRace(result);setGrid(first.grid);setMap(first.map_config);setAppliedMap(first.map_config);setAlgorithms(result.logs.map(log=>log.config.algorithm));setHeuristic(result.logs.find(log=>log.config.algorithm==='astar')?.config.heuristic??'manhattan');setStore(new RacePlayback(result.logs));setSelected(null);setMode('RACE');}
    catch(error){setError(error instanceof Error?error.message:String(error));}finally{setBusy(false);}
  };
  const changeMode=(next:Mode)=>{setMode(next);if(next==='BUILD')store?.pause();};
  const mapPending=JSON.stringify(map)!==JSON.stringify(appliedMap);
  return <div className="app"><header className="app-header"><a className="wordmark" href="#" aria-label="Project ARENA">ARENA</a><nav aria-label="Workspace mode">{(['BUILD','RACE','INSPECT','ANALYZE','BENCHMARK'] as Mode[]).map(tab=><button key={tab} aria-current={mode===tab?'page':undefined} className={mode===tab?'active':''} onClick={()=>changeMode(tab)}>{tab}</button>)}</nav><div className="file-actions"><button disabled={busy} onClick={()=>input.current?.click()}>Import Run</button><button disabled={!store||busy} onClick={()=>store&&exportRace(store.logs)}>↥ Export Run</button><input ref={input} type="file" hidden accept=".json,.arena.json" onChange={e=>{const file=e.target.files?.[0];if(file)void load(file);e.target.value='';}}/></div></header>
    <main><div className="title-row"><div><h1>Pathfinding arena</h1><p>Same problem. Same rules. Different search.</p></div><button className="primary" disabled={busy||!grid} onClick={run}>{busy?'Computing…':'▶ Run experiment'}</button></div>
    {error?<div role="alert" className="error">{error}</div>:null}
    {mapPending?<p className="notice">Map settings changed. Regenerate to apply them; the current grid uses seed {appliedMap.seed}.</p>:null}
    <LabControls busy={busy} map={map} expose={(config,pair)=>void scenario(config,pair)} showcase={(index,isDemo)=>void scenario(presets[index].map,presets[index].pair,isDemo)} lab={a=>void heuristicLab(a)}/><div className="workspace"><ConfigRail map={map} setMap={setMap} algorithms={algorithms} setAlgorithms={setAlgorithms} heuristic={heuristic} setHeuristic={setHeuristic} busy={busy} generate={()=>void generate()} randomize={()=>void generate({...map,seed:map.seed<Number.MAX_SAFE_INTEGER?map.seed+1:0})}/>
      <div className="main-surface">{mode==='BENCHMARK'?<Benchmark/>:!grid?<div className="empty-surface">{busy?'Loading Rust-generated problem…':'Rust engine unavailable. Review the error above.'}</div>:mode==='BUILD'?<section className="build-surface"><header><h2>Problem editor</h2><span>Seed {appliedMap.seed} · {grid.width} × {grid.height}</span></header><div className="editor-toolbar" role="toolbar" aria-label="Terrain tools">{tools.map(item=><button key={item} disabled={busy} className={tool===item?'selected':''} aria-pressed={tool===item} onClick={()=>setTool(item)}>{item==='mud'?'Mud · 3':item==='water'?'Water · 5':item==='heavy'?'Heavy · 10':item}</button>)}<button disabled={busy} onClick={()=>{setGrid(clearGrid(grid));resetRace();}}>Clear</button></div><div className="build-canvas-wrap"><GridCanvas grid={grid} store={null} selected={selected} editable onCell={onCell}/></div><p className="hint">Drag to paint. Start and goal stay traversable. Editing invalidates the previous run; the saved grid is authoritative.</p></section>:mode==='ANALYZE'?(store&&race?<Analysis store={store} features={race.features} observations={race.observations}/>:<div className="empty-surface">Run an experiment to measure and compare algorithms.</div>):<><RacePanels grid={grid} algorithms={algorithms} store={store} onCell={onCell} selected={selected} observations={race?.observations}/>{mode==='INSPECT'?(store?<Inspector grid={grid} store={store} node={selected}/>:<section className="inspector">Run an experiment, then select a cell to inspect its state.</section>):null}</>}
      </div></div>
      {store?<Timeline store={store}/>:<section className="playback empty-playback"><h2>Playback</h2><p>Run an experiment to create a real event log.</p><span>0 / — events</span></section>}
      <footer className="legend"><h2>Legend</h2>{[['Expanded','expanded'],['Frontier','frontier'],['Current','current'],['Path','path'],['Mud 3','mud'],['Water 5','water'],['Heavy 10','heavy'],['Wall','wall']].map(([label,id])=><span key={id}><i className={id}/>{label}</span>)}<small>Normal terrain = 1 · walls impassable</small></footer>
    </main></div>;
}
