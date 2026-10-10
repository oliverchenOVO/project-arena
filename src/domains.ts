import {request} from './backend';
import {sameJson} from './jsonContract';
export type ArenaDomain='pathfinding'|'sorting'|'graph';
export interface DomainEvent {schemaVersion:2;domain:ArenaDomain;runId:string;algorithmId:string;tick:number;sequence:number;eventType:string;payload:Record<string,unknown>}
export interface SortConfig {length:number;min:number;max:number;distribution:string;seed:number}
export interface GraphNode {x:number;y:number}
export interface GraphEdge {from:number;to:number;weight:number}
export interface GraphProblem {nodes:GraphNode[];edges:GraphEdge[];directed:boolean;weighted:boolean;start:number;goal:number;config:Record<string,unknown>}
export type DomainProblem={values:number[];config:SortConfig}|GraphProblem;
export interface DomainLog {schemaVersion:2;domain:ArenaDomain;algorithmId:string;algorithmVersion:string;problem:DomainProblem;events:DomainEvent[];metrics:Record<string,number|boolean|null>;canonicalHash:string;computeNs:number}
export interface DomainResult {logs:DomainLog[];features?:Record<string,unknown>;observations?:Record<string,unknown>}
export const domainBackend={
 generate:(domain:ArenaDomain,config:unknown)=>request<{problem:DomainProblem}>({action:'domain_generate',domain,config}),
 race:(domain:ArenaDomain,problem:DomainProblem,algorithms:string[])=>request<DomainResult>({action:'domain_race',domain,problem,algorithms}),
 validate:(logs:DomainLog[])=>request<DomainResult>({action:'domain_validate',logs}),
};
export function downloadRun(logs:DomainLog[]) {const data={format:'arena-run',version:2,domain:logs[0].domain,problem:logs[0].problem,algorithms:logs.map(l=>l.algorithmId),logs};const url=URL.createObjectURL(new Blob([JSON.stringify(data)],{type:'application/json'}));const a=document.createElement('a');a.href=url;a.download=`arena-${logs[0].domain}.arena.json`;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}
export async function readDomainRun(file:File) {if(file.size>128*1024*1024)throw Error('Run file exceeds 128 MiB');const data=JSON.parse(await file.text());if(data.format!=='arena-run'||data.version!==2||!['sorting','graph'].includes(data.domain)||!Array.isArray(data.logs)||!sameJson(data.algorithms,data.logs.map((l:DomainLog)=>l.algorithmId))||data.logs.some((l:DomainLog)=>l.domain!==data.domain||!sameJson(l.problem,data.problem)))throw Error('Unsupported ARENA race schema');return domainBackend.validate(data.logs);}
