import {useSyncExternalStore} from 'react';
import {speeds} from '../playback';
import type {RacePlayback,Speed} from '../playback';
export default function Timeline({store}:{store:RacePlayback}) {
  useSyncExternalStore(store.subscribe,store.snapshot);
  return <section className="playback"><h2>Playback</h2><div className="transport">
    <button title="Beginning" aria-label="Beginning" onClick={()=>{store.pause();store.seek(0);}}>⏮</button>
    <button title="Step backward" aria-label="Step backward" onClick={()=>store.stepEvent(-1)}>‹</button>
    <button className={store.playing?'selected':''} aria-label={store.playing?'Pause':'Play'} onClick={()=>store.toggle()}>{store.playing?'Ⅱ':'▶'}</button>
    <button title="Step event" aria-label="Step event" onClick={()=>store.stepEvent(1)}>›</button>
    <button title="Step expansion" aria-label="Step expansion" onClick={()=>store.stepExpansion()}>»</button>
    <button title="End" aria-label="End" onClick={()=>{store.pause();store.seek(store.length);}}>⏭</button>
  </div><label className="speed-label">Speed<select aria-label="Playback speed" value={store.speed} onChange={e=>store.setSpeed(e.target.value==='MAX'?'MAX':Number(e.target.value) as Speed)}>{speeds.map(speed=><option key={speed} value={speed}>{speed==='MAX'?'MAX':`${speed}×`}</option>)}</select></label>
  <input aria-label="Event timeline" type="range" min="0" max={store.length} value={store.cursor} onChange={e=>{store.pause();store.seek(Number(e.target.value));}}/>
  <output>{store.cursor.toLocaleString()} / {store.length.toLocaleString()} events</output><p className="clock-note">Shared event clock · 1× = 1,000 events/s · » step expansion</p></section>;
}
