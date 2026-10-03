import type {Grid,MapConfig,RaceResult,RunConfig,RunLog} from './model';
async function request<T>(payload:unknown):Promise<T> {
  if('__TAURI_INTERNALS__' in window) {
    const {invoke}=await import('@tauri-apps/api/core');
    return invoke<T>('arena_request',{payload});
  }
  const response=await fetch('/api/arena',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(payload)});
  const body=await response.json();if(!response.ok)throw new Error(body.error||'Rust engine request failed');return body as T;
}
export const backend={
  generate:(map_config:MapConfig)=>request<{grid:Grid}>({action:'generate',map_config}),
  race:(grid:Grid,map_config:MapConfig,configs:RunConfig[])=>request<RaceResult>({action:'race',grid,map_config,configs}),
  validate:(logs:RunLog[])=>request<RaceResult>({action:'validate',logs}),
};
export function exportRace(logs:RunLog[]):void {
  const data={schema_version:1,kind:'arena_race',logs};
  const url=URL.createObjectURL(new Blob([JSON.stringify(data)],{type:'application/json'}));
  const a=document.createElement('a');a.href=url;a.download=`arena-${logs[0].map_config.seed}.arena.json`;a.click();URL.revokeObjectURL(url);
}
export async function importRace(file:File):Promise<RaceResult> {
  if(file.size>128*1024*1024)throw new Error('Run file exceeds 128 MiB');
  const data:unknown=JSON.parse(await file.text());
  if(!data||typeof data!=='object'||!('schema_version' in data)||data.schema_version!==1||!('logs' in data)||!Array.isArray(data.logs))throw new Error('Unsupported ARENA race schema');
  return backend.validate(data.logs as RunLog[]);
}
