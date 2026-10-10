import fs from 'node:fs';import {spawnSync} from 'node:child_process';
function bridge(payload){const r=spawnSync('target/release/arena.exe',['bridge'],{input:JSON.stringify(payload),encoding:'utf8',maxBuffer:64*1024*1024});if(r.status!==0)throw Error(r.stderr);return JSON.parse(r.stdout);}
const specs=[['sorting',{length:32,min:1,max:100,distribution:'random',seed:123},['quick','merge']],['sorting',{length:32,min:1,max:100,distribution:'nearly_sorted',seed:123},['insertion','quick']],['graph',{size:32,seed:123,family:'sparse',directed:false,weighted:false},['bfs','dfs']],['graph',{size:32,seed:123,family:'bridge',directed:false,weighted:true},['prim','kruskal']]];
const fixtures=specs.map(([domain,config,algorithms])=>{const {problem}=bridge({action:'domain_generate',domain,config});return bridge({action:'domain_race',domain,problem,algorithms});});
fs.mkdirSync('src/fixtures',{recursive:true});fs.writeFileSync('src/fixtures/domain-showcase.json',JSON.stringify(fixtures)+'\n');
