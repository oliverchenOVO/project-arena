import {useEffect,useMemo,useRef,useState,useSyncExternalStore} from 'react';
import {useLocale} from '../i18n';
import {presets,eventMarkers} from '../laboratory';
import {metric,names} from '../model';
import type {RunLog} from '../model';
import type {RacePlayback} from '../playback';

export function Landing({busy,enter,start}:{busy:boolean;enter:()=>void;start:(index:number,demo:boolean)=>void}){
 const{t}=useLocale();
 return <main className="landing"><section className="landing-hero"><div className="eyebrow">PROJECT ARENA</div><h1>{t('See how algorithms think.')}</h1><p>{t('Same problem, rules and Seed. A fair race.')}</p><div className="button-row"><button className="primary" disabled={busy} onClick={()=>start(0,true)}>{t('Start the 60-second demo')}</button><button onClick={enter}>{t('Enter laboratory')}</button></div><div className="hero-grid" aria-hidden="true"><span>A*</span><i/><span>Dijkstra</span><small>g + h · N → E → S → W</small></div></section><section aria-label={t('Recommended experiments')}><h2>{t('Recommended experiments')}</h2><div className="showcase-cards">{presets.map((preset,index)=><button key={preset.title} className="showcase-card" disabled={busy} onClick={()=>start(index,false)}><span className="card-number">0{index+1}</span><h3>{t(preset.title)}</h3><p>{t(preset.objective)}</p><small>{preset.pair.map(a=>names[a]).join(' / ')} · Seed {preset.map.seed}</small></button>)}</div></section></main>;
}

export function QuickStart(){
 const{t}=useLocale();const[visible,setVisible]=useState(()=>{try{return localStorage.getItem('arena.guide.v1')!=='dismissed';}catch{return true;}});
 if(!visible)return null;
 return <section className="quick-start" aria-label={t('Quick start')}><strong>{t('Quick start')}</strong><ol><li>{t('Choose a problem')}</li><li>{t('Choose contenders')}</li><li>{t('Run and observe')}</li></ol><button onClick={()=>setVisible(false)}>{t('Dismiss')}</button><button onClick={()=>{try{localStorage.setItem('arena.guide.v1','dismissed');}catch{/* session still dismisses */}setVisible(false);}}>{t('Do not show again')}</button></section>;
}

export function GuidedDemo({store,restart}:{store:RacePlayback;restart:()=>void}){
 const{t}=useLocale();useSyncExternalStore(store.subscribe,store.snapshot);
 const stage=useRef(0);const[callout,setCallout]=useState(false);
 const stops=useMemo(()=>eventMarkers(store.logs[0]).filter(m=>['Cost update milestone','Goal discovered'].includes(m.label)).slice(0,3),[store]);
 useEffect(()=>store.subscribe(()=>{
  const stop=stops[stage.current];
  if(stop&&store.playing&&store.replays[0].cursor>=stop.cursor&&store.cursor<store.length){stage.current++;store.pause();store.jump(0,stop.cursor);setCallout(true);}
 }),[store,stops]);
 const state=store.replays[0].state;
 return <section className="demo-callout" aria-live="polite"><div><strong>{t('Guided demo')} · {Math.min(stage.current+1,4)} / 4</strong><p>{t('At event {event}, {algorithm} has expanded {expanded} nodes; frontier {frontier}.',{event:store.replays[0].cursor,algorithm:names[store.logs[0].config.algorithm],expanded:state.metrics.expanded,frontier:state.frontier})}</p>{callout?<small>{t('Recorded milestone')}</small>:null}</div>{callout?<button className="primary" onClick={()=>{setCallout(false);store.toggle();}}>{t('Continue demo')}</button>:null}<button onClick={restart}>{t('Restart demo')}</button></section>;
}

export function analysisHeadline(logs:RunLog[]):{key:string;values:Record<string,string|number>}{
 const a=logs.find(l=>l.config.algorithm==='astar'),d=logs.find(l=>l.config.algorithm==='dijkstra');
 if(a&&d&&d.metrics.expanded>0&&a.metrics.path_cost!==null&&a.metrics.path_cost===d.metrics.path_cost&&a.metrics.expanded<=d.metrics.expanded)return {key:'Same optimal cost; A* expanded {reduction}% fewer nodes.',values:{reduction:((1-a.metrics.expanded/d.metrics.expanded)*100).toFixed(1)}};
 return {key:'Compare the recorded metrics independently.',values:{}};
}

export function AnalysisOverview({logs}:{logs:RunLog[]}){
 const{t}=useLocale();const headline=analysisHeadline(logs);
 return <section className="analysis-overview"><h2>{t(headline.key,headline.values)}</h2><p>{t('Lower path cost is better. Each chart uses its own unit.')}</p><div className="metric-charts">{(['expanded','logical_ops','peak_frontier','path_cost'] as const).map((key,index)=>{const title=['Expanded','Logical ops','Peak frontier','Path cost'][index];const unit=['nodes','operations','entries','cost units'][index];const maximum=Math.max(1,...logs.map(l=>l.metrics[key]??0));return <section key={key} aria-label={t(title)}><h3>{t(title)} <small>({t(unit)})</small></h3>{logs.map((log,i)=><div className="metric-bar" key={i}><span>{names[log.config.algorithm]}</span><div><i style={{width:`${100*(log.metrics[key]??0)/maximum}%`}}/></div><b>{metric(log.metrics[key])}</b></div>)}</section>;})}</div><ol className="conclusions"><li>{t('Equal cost does not imply equal search effort.')}</li><li>{t('The heuristic result depends on this map and Seed.')}</li><li>{t('Replay can revisit each recorded decision.')}</li></ol></section>;
}
