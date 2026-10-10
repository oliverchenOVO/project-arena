import type {Grid,MapConfig,RaceResult,RunConfig,RunLog} from './model';
import {sameJson} from './jsonContract';
export async function request<T>(payload:unknown):Promise<T> {
  if('__TAURI_INTERNALS__' in window) {
    const {invoke}=await import('@tauri-apps/api/core');
    return invoke<T>('arena_request',{payload});
  }
  const response=await fetch('/api/arena',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(payload)});
  const body=await response.json();if(!response.ok)throw new Error(String(body.error||'Rust engine request failed').replace(/^ARENA error:\s*/,'').trim());return body as T;
}
export const backend={
  generate:(map_config:MapConfig)=>request<{grid:Grid}>({action:'generate',map_config}),
  race:(grid:Grid,map_config:MapConfig,configs:RunConfig[])=>request<RaceResult>({action:'race',grid,map_config,configs}),
  validate:(logs:RunLog[])=>request<RaceResult>({action:'validate',logs}),
};
export function exportRace(logs:RunLog[]):void {
  const data={format:'arena-run',version:2,domain:'pathfinding',problem:{grid:logs[0].grid,map_config:logs[0].map_config},algorithms:logs.map(l=>l.config),logs};
  const url=URL.createObjectURL(new Blob([JSON.stringify(data)],{type:'application/json'}));
  const a=document.createElement('a');a.href=url;a.download=`arena-${logs[0].map_config.seed}.arena.json`;document.body.append(a);a.click();a.remove();
  // Let the browser start consuming the blob before releasing its object URL.
  window.setTimeout(()=>URL.revokeObjectURL(url),1000);
}
export async function importRace(file:File):Promise<RaceResult> {
  if(file.size>128*1024*1024)throw new Error('Run file exceeds 128 MiB');
  const data:unknown=JSON.parse(await file.text());
  if(!data||typeof data!=='object'||!('logs' in data)||!Array.isArray(data.logs)||!(('schema_version' in data&&data.schema_version===1)||('format' in data&&data.format==='arena-run'&&'version' in data&&data.version===2&&'domain' in data&&data.domain==='pathfinding')))throw new Error('Unsupported ARENA race schema');
  if('version'in data&&data.version===2){const d=data as {problem?:unknown;algorithms?:unknown;logs:RunLog[]};if(!d.logs[0]||!sameJson(d.problem,{grid:d.logs[0].grid,map_config:d.logs[0].map_config})||!sameJson(d.algorithms,d.logs.map(l=>l.config)))throw new Error('Unsupported ARENA race schema');}
  return backend.validate(data.logs as RunLog[]);
}
