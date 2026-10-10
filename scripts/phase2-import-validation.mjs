import fs from 'node:fs';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const fixtures=JSON.parse(fs.readFileSync('src/fixtures/domain-showcase.json'));
const sorted=v=>Array.isArray(v)?v.map(sorted):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).sort().map(k=>[k,sorted(v[k])])):v;
const seal=log=>{const {schemaVersion,domain,algorithmId,algorithmVersion,problem,events,metrics}=log;log.canonicalHash=createHash('sha256').update(JSON.stringify(sorted({schemaVersion,domain,algorithmId,algorithmVersion,problem,events,metrics}))).digest('hex');};
const cases=[];
for(const fixture of fixtures)for(const kind of ['valid','timing-only','invalid-hash','rehashed-metrics','rehashed-run-id']){
 const logs=structuredClone(fixture.logs);
 if(kind==='timing-only')logs[0].computeNs=0;
 if(kind==='invalid-hash')logs[0].canonicalHash='0'.repeat(64);
 if(kind==='rehashed-metrics'){logs[0].metrics.logicalOps++;seal(logs[0]);}
 if(kind==='rehashed-run-id'){logs[0].events[0].runId='forged';seal(logs[0]);}
 const r=spawnSync('target/release/arena.exe',['bridge'],{input:JSON.stringify({action:'domain_validate',logs}),encoding:'utf8',maxBuffer:64*1024*1024});
 const expected=['valid','timing-only'].includes(kind);if((r.status===0)!==expected)throw Error(`${kind}: ${r.stderr}`);
 cases.push({domain:logs[0].domain,algorithm:logs[0].algorithmId,kind,accepted:r.status===0,expected});
}
fs.writeFileSync('phase2-import-verification.json',JSON.stringify({cases,checks:cases.length,failures:0},null,2)+'\n');
console.log(`v2 import: ${cases.length} checks PASS`);
