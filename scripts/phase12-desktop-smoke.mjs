import {chromium,expect} from '@playwright/test';
import {readFileSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
const dictionaries=Object.fromEntries(['en','zh-TW'].map(l=>[l,JSON.parse(readFileSync(`src/i18n/${l}.json`,'utf8'))]));
const browser=await chromium.connectOverCDP(process.env.ARENA_CDP_URL||'http://127.0.0.1:9228',{timeout:60000});
const page=browser.contexts()[0].pages()[0];page.setDefaultTimeout(30000);
const errors=[],remoteRequests=[],cases=[],screenshots=[];page.on('pageerror',e=>errors.push(e.message));page.on('request',r=>{if(r.url().startsWith('https:'))remoteRequests.push(r.url());});
const t=(key,locale)=>dictionaries[locale][key]??key;
const button=(key,locale)=>page.getByRole('button',{name:t(key,locale),exact:true});
const label=(key,locale)=>page.getByLabel(t(key,locale),{exact:true});
const capture=async(name)=>{const path=`docs/screenshots/phase12-${name}.png`;await page.screenshot({path,fullPage:true});screenshots.push(path);};
await page.getByRole('button',{name:'繁中',exact:true}).waitFor();
for(const locale of ['zh-TW','en']){
 await page.getByRole('button',{name:locale==='en'?'EN':'繁中',exact:true}).click();await page.getByRole('link',{name:'Project ARENA'}).click();
 if(locale==='zh-TW')await capture('zh-landing');
 for(let index=0;index<5;index++){
  await page.locator('.showcase-card').nth(index).click();await label('Event timeline',locale).waitFor();await label('End',locale).click();
  const final=await page.locator('.race-metrics b').allTextContents();await label('Beginning',locale).click();await label('Step expansion',locale).click();await label('End',locale).click();const replayMatches=JSON.stringify(final)===JSON.stringify(await page.locator('.race-metrics b').allTextContents());
  const guide=button('Do not show again',locale);if(await guide.isVisible())await guide.click();
  if(index===0){await capture(locale==='en'?'en-race':'zh-astar-dijkstra-race');if(locale==='zh-TW'){await button('Presentation mode',locale).click();await capture('windows-packaged');await page.keyboard.press('Escape');}await button('Overlay Diff',locale).click();if(locale==='zh-TW')await capture('zh-overlay-diff');await button('Split View',locale).click();}
  await button('INSPECT',locale).click();const canvas=label('Algorithm execution grid',locale).first();const box=await canvas.boundingBox();await canvas.click({position:{x:box.width/32*(index===3?16.5:1.5),y:box.height/32*(index===3?16.5:1.5)}});const inspector=await page.locator('.node-details').isVisible();if(locale==='zh-TW'&&index===0)await capture('zh-inspector-astar-dijkstra');if(locale==='zh-TW'&&index===3)await capture('zh-inspector-bfs-dijkstra');
  await button('ANALYZE',locale).click();await page.getByRole('heading',{name:t('Measured execution',locale),exact:true}).waitFor();await page.locator('.analysis>details').last().locator('summary').click();const manifests=await page.locator('.analysis>details').last().locator('pre').allTextContents();const hashes=manifests.map(m=>JSON.parse(m).hash);const charts=await page.locator('.metric-charts>section').count();
  if(locale==='zh-TW'&&index===0){await page.locator('.analysis>details').last().locator('summary').click();await capture('zh-analysis');}
  cases.push({locale,scenario:index,replayMatches,inspector,charts,hashes});await page.getByRole('link',{name:'Project ARENA'}).click();
 }
 await button('Enter laboratory',locale).click();await button('RACE',locale).click();await button('Show configuration',locale).click();await page.locator('.lab-controls>summary').click();
 if(locale==='zh-TW')await capture('zh-stress-lab');await label('Stress target',locale).selectOption('dfs');await button('Expose weakness',locale).click();await expect(page.getByRole('button',{name:t('Run experiment',locale)})).toBeEnabled({timeout:60000});await label('End',locale).click();
 await button('Show configuration',locale).click();await button('Compare four heuristics',locale).click();await expect(page.getByRole('button',{name:t('Run experiment',locale)})).toBeEnabled({timeout:60000});await label('End',locale).click();const heuristicPanes=await label('Algorithm execution grid',locale).count();if(heuristicPanes!==5)throw Error('Heuristic lab missing actual reference');
 // Native file input validates a saved engine log without another execution.
 const log=JSON.parse(readFileSync('src/__tests__/fixtures/weighted-run.json','utf8'));await page.locator('.file-actions input[type=file]').setInputFiles({name:'fixture.arena.json',mimeType:'application/json',buffer:Buffer.from(JSON.stringify({schema_version:1,kind:'arena_race',logs:[log]}))});await expect(label('Algorithm execution grid',locale)).toHaveCount(1);await label('Event timeline',locale).waitFor();await label('Beginning',locale).click();await label('End',locale).click();
 await button('BENCHMARK',locale).click();await page.locator('.analysis input[type=file]').setInputFiles(resolve('benchmark.json'));await page.getByRole('heading',{name:t('Expanded-node heatmap · mean per algorithm × family',locale),exact:true}).waitFor();if(locale==='zh-TW')await capture('zh-benchmark');
 cases.push({locale,heuristicPanes,importValidated:true,benchmarkLoaded:true});await page.getByRole('link',{name:'Project ARENA'}).click();
}
const manifest=JSON.parse(readFileSync('output/windows/artifact-manifest.json','utf8').replace(/^\uFEFF/,''));
const viewport=await page.evaluate(()=>({width:innerWidth,height:innerHeight,devicePixelRatio}));const report={version:'0.1.2',viewport,tauri:await page.evaluate(()=> '__TAURI_INTERNALS__'in window),url:page.url(),title:await page.title(),freshProfile:true,portableCopyVerified:true,artifacts:manifest,cases,screenshots,consoleErrors:errors,remoteRequests};
writeFileSync('windows-phase12-verification.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report,null,2));await browser.close();if(!report.tauri||errors.length||remoteRequests.length||cases.some(c=>c.replayMatches===false||c.charts&&c.charts!==4))process.exit(1);
