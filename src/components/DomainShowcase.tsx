import {useMemo} from 'react';
import fixture from '../fixtures/domain-showcase.json';
import type {ArenaDomain,DomainResult} from '../domains';
import {DomainReplay} from '../domainPlayback';
import DomainCanvas from './DomainCanvas';
import {useLocale} from '../i18n';
export default function DomainShowcase({enter}:{enter:(domain:ArenaDomain)=>void}) {const {t}=useLocale();const previews=useMemo(()=>(fixture as unknown as DomainResult[]).map(r=>{const replay=new DomainReplay(r.logs[0]);replay.seek(Math.floor(r.logs[0].events.length*.6));return replay;}),[]);return <section className="domain-showcase"><h2>{t('Explore sorting and graphs')}</h2><p>{t('Previews replay saved Rust executions. Open a domain to run your own experiment.')}</p><div className="domain-panels">{previews.map((r,i)=><article key={i} className="race-panel"><header><h3>{(fixture as unknown as DomainResult[])[i].logs.map(l=>t(l.algorithmId)).join(' / ')}</h3><button onClick={()=>enter(r.log.domain)}>{t('Open domain')}</button></header><DomainCanvas log={r.log} state={r.state} revision={0} heapTree={false} onSelect={()=>enter(r.log.domain)}/><p className="hint">{t('Recorded preview')} · #{r.cursor} · {r.log.canonicalHash.slice(0,16)}</p></article>)}</div></section>;}
