import {heuristicNames} from '../i18n/terminology';
import {useLocale} from '../i18n';
import {useMemo,useState,useSyncExternalStore} from 'react';
import {speeds} from '../playback';
import type {RacePlayback,Speed} from '../playback';
import type {SyncMode} from '../playback';
import {eventMarkers} from '../laboratory';
import {names} from '../model';
import type {DomainPlayback} from '../domainPlayback';
import DomainTimeline from './DomainTimeline';
export default function Timeline({store}:{store:RacePlayback|DomainPlayback}) {
  return store.logs[0]&&'domain' in store.logs[0]?<DomainTimeline store={store as DomainPlayback}/>:<PathTimeline store={store as RacePlayback}/>;
}
function PathTimeline({store}:{store:RacePlayback}) {
  const {t}=useLocale();
  const [position,setPosition]=useState('');
  useSyncExternalStore(store.subscribe,store.snapshot);
  const markers=useMemo(()=>store.logs.map(eventMarkers),[store]);
  return <section className="playback"><h2>{t("Playback")}</h2><div className="transport">
    <button title={t("Beginning")} aria-label={t("Beginning")} onClick={()=>{store.pause();store.seek(0);}}>⏮</button>
    <button title={t("Step backward")} aria-label={t("Step backward")} onClick={()=>store.stepEvent(-1)}>‹</button>
    <button className={store.playing?'selected':''} aria-label={t(store.playing?'Pause':'Play')} onClick={()=>store.toggle()}>{store.playing?'Ⅱ':'▶'}</button>
    <button title={t("Step event")} aria-label={t("Step event")} onClick={()=>store.stepEvent(1)}>›</button>
    <button title={t("Step expansion")} aria-label={t("Step expansion")} onClick={()=>store.stepExpansion()}>»</button>
    <button title={t("End")} aria-label={t("End")} onClick={()=>{store.pause();store.seek(store.length);}}>⏭</button>
  </div><label className="speed-label">{t("Speed")}<select aria-label={t("Playback speed")} value={store.speed} onChange={e=>store.setSpeed(e.target.value==='MAX'?'MAX':Number(e.target.value) as Speed)}>{speeds.map(speed=><option key={speed} value={speed}>{speed==='MAX'?'MAX':`${speed}×`}</option>)}</select></label>
  <input aria-label={t("Event timeline")} type="range" min="0" max={store.length} value={store.cursor} onChange={e=>{store.pause();store.seek(Number(e.target.value));}}/>
  <output>{store.cursor.toLocaleString()} / {store.length.toLocaleString()}{t("clock units")}</output><label className="sync-label">{t("Compare synchronization")}<select aria-label={t("Compare synchronization")} value={store.syncMode} onChange={e=>store.setSync(e.target.value as SyncMode)}><option value="event">{t("Event Index Sync")}</option><option value="expansion">{t("Expansion Count Sync")}</option><option value="percentage">{t("Percentage Sync")}</option></select></label><p className="clock-note">{t("Event positions are not equal algorithmic work. Percentage maps normalized progress to each log; Expansion Count compares the same expansion count. 1× = 1,000 clock units/s.")}</p><div className="timeline-markers">{markers.map((group,i)=><div className="marker-track" key={i} aria-label={t('{algorithm} marker track',{algorithm:names[store.logs[i].config.algorithm]})}>{group.map((m,j)=><button key={j} aria-label={t("{algorithm} {label} marker #{event}",{algorithm:names[store.logs[i].config.algorithm],label:t(m.label),event:m.event})} title={`${t(m.label)} #${m.event}`} style={{left:`${100*(store.syncMode==='event'?m.cursor/store.length:store.syncMode==='percentage'?m.cursor/store.logs[i].events.length:store.expansionCursors[i].filter(c=>c<=m.cursor).length/Math.max(...store.expansionCursors.map(c=>c.length)))}%`}} onClick={()=>store.jump(i,m.cursor)}>◆</button>)}</div>)}</div><details className="precise-navigation"><summary>{t('Precise navigation and keyboard shortcuts')}</summary><form className="position-selector" onSubmit={event=>{event.preventDefault();const target=Number(position);if(position!==''&&Number.isInteger(target)&&target>=0&&target<=store.length){store.pause();store.seek(target);}}}><label>{t('Clock position')}<input aria-label={t('Clock position')} type="number" min={0} max={store.length} step={1} required placeholder={String(store.cursor)} value={position} onChange={e=>setPosition(e.target.value)}/></label><button type="submit">{t('Jump to position')}</button></form><p>{t('Jump uses the selected synchronization mode; the clock position is not an event ID for every algorithm.')}</p><p>{t('Space: play/pause · Left/Right: one clock unit · Shift+Right: next expansion · Home/End: beginning/end.')}</p><p>{t('Shortcuts are inactive while typing, using controls, editing a map, or following the guided demo.')}</p></details><details className="marker-details"><summary>{t("Event markers · click to jump")}</summary>{markers.map((group,index)=><div className="marker-row" key={index}><h3>{names[store.logs[index].config.algorithm]} · {t(heuristicNames[store.logs[index].config.heuristic])}</h3>{group.map((m,i)=><button key={i} title={`${t("Recorded event")} #${m.event}`} onClick={()=>store.jump(index,m.cursor)}>{t(m.label)} #{m.event}</button>)}</div>)}</details></section>;
}
