import {createRequire} from 'node:module';
import {resolve} from 'node:path';
import {writeFile,readFile} from 'node:fs/promises';
const require=createRequire(resolve('package.json'));
const {chromium,expect}=require('@playwright/test');
const browser=await chromium.connectOverCDP(process.env.ARENA_CDP_URL||'http://127.0.0.1:9223',{timeout:60000});
const page=browser.contexts()[0].pages()[0];const errors=[],remoteRequests=[];
page.on('pageerror',e=>errors.push(e.message));page.on('request',r=>{if(r.url().startsWith('https:'))remoteRequests.push(r.url());});await page.route('https://**',r=>r.abort());
const ready=()=>expect(page.getByRole('button',{name:'Run experiment'})).toBeEnabled({timeout:60000});
const lab=async()=>{const d=page.locator('.lab-controls');if(!await d.evaluate(e=>e.open))await d.locator('summary').first().click();};
const hashes=async()=>{await page.getByRole('button',{name:'ANALYZE',exact:true}).click();const manifest=page.locator('.analysis>details').last();if(!await manifest.evaluate(e=>e.open))await manifest.locator('summary').click();return (await manifest.locator('pre').allTextContents()).map(t=>JSON.parse(t).hash);};
await ready();const cases=[];
for(let index=0;index<5;index++){
 await lab();await page.getByLabel('Showcase preset').selectOption(String(index));await page.getByRole('button',{name:'Run showcase',exact:true}).click();await ready();await page.getByLabel('End',{exact:true}).click();
 const final=await page.locator('.race-metrics').allTextContents();const canonicalHashes=await hashes();
 await page.getByRole('button',{name:'RACE',exact:true}).click();await page.getByRole('button',{name:'Overlay Diff',exact:true}).click();await expect(page.getByRole('heading',{name:'GHOST OVERLAY'})).toBeVisible();await expect(page.locator('.diff-summary')).toContainText('Shared:');
 await page.getByLabel('Beginning',{exact:true}).click();await page.getByLabel('Step expansion',{exact:true}).click();await page.getByLabel('End',{exact:true}).click();await page.getByRole('button',{name:'Split View',exact:true}).click();expect(await page.locator('.race-metrics').allTextContents()).toEqual(final);
 await page.getByRole('button',{name:'INSPECT',exact:true}).click();const canvas=page.getByLabel('Algorithm execution grid').first();const rect=await canvas.boundingBox();await canvas.click({position:{x:rect.width*1.5/32,y:rect.height*1.5/32}});await expect(page.locator('.node-details')).toContainText('Parent changed');await page.getByRole('button',{name:'ANALYZE',exact:true}).click();await expect(page.getByRole('heading',{name:/DERIVED · stress effect/})).toBeVisible();
 cases.push({preset:index,canonicalHashes,replayMatches:true,ghost:true,inspector:true,analysis:true,final});
}
await lab();await page.getByLabel('Stress target').selectOption('dfs');await page.getByRole('button',{name:'Expose weakness',exact:true}).click();await ready();await expect(page.getByLabel('Map family')).toHaveValue('dfs_deep_trap');await page.getByLabel('End',{exact:true}).click();
await lab();await page.getByRole('button',{name:'Compare four heuristics'}).click();await ready();await expect(page.getByLabel('Algorithm execution grid')).toHaveCount(5);await page.getByLabel('End',{exact:true}).click();await page.getByRole('button',{name:'ANALYZE',exact:true}).click();await expect(page.locator('.analysis')).toContainText('A* / chebyshev');
await page.getByLabel('Compare synchronization').selectOption('expansion');await page.locator('.marker-details>summary').click();await page.locator('.marker-row').first().getByRole('button',{name:/Goal expanded/}).click();await page.getByLabel('End',{exact:true}).click();
await page.getByRole('button',{name:'BENCHMARK',exact:true}).click();await page.locator('.analysis input[type=file]').setInputFiles(resolve('stress-benchmark.json'));await expect(page.getByRole('heading',{name:/Expanded-node heatmap/})).toBeVisible();
const demoHashes=[];
for(let i=0;i<2;i++){await lab();await page.getByLabel('Showcase preset').selectOption('3');await page.getByRole('button',{name:'Demo mode / restart'}).click();await ready();await page.getByLabel('Playback speed').selectOption('MAX');await expect(page.getByRole('heading',{name:/RACE COMPLETE · Observation/})).toBeVisible();demoHashes.push(await hashes());}
expect(demoHashes[0]).toEqual(demoHashes[1]);
await page.getByRole('button',{name:'RACE',exact:true}).click();await page.getByRole('button',{name:'Overlay Diff',exact:true}).click();const panel=page.locator('.lab-controls');if(await panel.evaluate(e=>e.open))await panel.locator('summary').first().click();await page.screenshot({path:'docs/screenshots/phase11-windows-ghost.png',fullPage:true});
const report={version:'windows-phase11/1',productVersion:'0.1.1',url:page.url(),title:await page.title(),tauri:await page.evaluate(()=> '__TAURI_INTERNALS__' in window),showcase:cases,exposeWeakness:true,heuristicLab:true,markers:true,benchmarkViewer:true,demoHashes,restartHashesMatch:true,consoleErrors:errors,remoteRequests,baselineSmoke:JSON.parse(await readFile('windows-launch-verification.json','utf8'))};
await writeFile('windows-phase11-verification.json',JSON.stringify(report,null,2));console.log(JSON.stringify(report,null,2));await browser.close();if(!report.tauri||errors.length||remoteRequests.length)process.exit(1);
