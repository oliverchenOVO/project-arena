import {ui} from './ui';
import {test,expect} from '@playwright/test';

test.beforeEach(async({page})=>{await page.addInitScript(()=>localStorage.setItem('arena.locale.v1','zh-TW'));await page.goto('/');await page.getByRole('button',{name:ui('Enter laboratory'),exact:true}).click();await expect(page.getByRole('button',{name:ui("Run experiment")})).toBeEnabled({timeout:60000});});
test('identity, editor validity, drag painting and clear',async({page})=>{
  await expect(page).toHaveTitle(/Project ARENA/);await expect(page.getByRole('heading',{name:ui("Pathfinding arena")})).toBeVisible();
  await page.getByRole('button',{name:ui("BUILD"),exact:true}).click();
  const canvas=page.getByLabel(ui("Editable problem grid"));const box=(await canvas.boundingBox())!;
  await page.getByRole('button',{name:ui("Heavy · 10"),exact:false}).click();
  await page.mouse.move(box.x+box.width*.2,box.y+box.height*.2);await page.mouse.down();await page.mouse.move(box.x+box.width*.5,box.y+box.height*.2,{steps:20});await page.mouse.up();
  await page.getByRole('button',{name:ui("Clear"),exact:true}).click();
  await page.getByLabel(ui("Width"),{exact:true}).fill('0');await page.getByRole('button',{name:ui("Regenerate"),exact:false}).click();await expect(page.getByRole('alert')).toContainText(ui('Dimensions must be integers from 1 to 128'));
  await page.getByLabel(ui("Width"),{exact:true}).fill('16');await page.getByLabel(ui("Height"),{exact:true}).fill('16');await page.getByRole('button',{name:ui("Regenerate"),exact:false}).click();await expect(page.getByRole('alert')).toHaveCount(0);
});
for(const [a,b] of [['bfs','dfs'],['dijkstra','astar'],['astar','greedy']])for(const family of ['open','backtracker','weighted','corridors','obstacles']) {
  test(`${a} vs ${b} / ${family}: race, replay, inspect and analysis`,async({page})=>{
    const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
    await page.getByLabel(ui("Width"),{exact:true}).fill('16');await page.getByLabel(ui("Height"),{exact:true}).fill('16');await page.getByLabel(ui("Map family")).selectOption(family);await page.getByLabel('演算法 A',{exact:true}).selectOption(a);await page.getByLabel('演算法 B',{exact:true}).selectOption(b);await page.getByRole('button',{name:ui("Regenerate"),exact:false}).click();
    await expect(page.getByRole('button',{name:ui("Run experiment")})).toBeEnabled();await page.getByRole('button',{name:ui("Run experiment")}).click();
    await expect(page.getByLabel(ui("Event timeline"))).toBeVisible();await page.getByLabel(ui("End"),{exact:true}).click();const finalText=await page.locator('.race-metrics').allTextContents();
    await page.getByLabel(ui("Beginning"),{exact:true}).click();await page.getByLabel(ui("Step expansion"),{exact:true}).click();await page.getByLabel(ui("Step backward"),{exact:true}).click();await page.getByLabel(ui("Step event"),{exact:true}).click();await page.getByLabel(ui("Playback speed")).selectOption('8');await page.getByLabel(ui("End"),{exact:true}).click();expect(await page.locator('.race-metrics').allTextContents()).toEqual(finalText);
    await page.getByRole('button',{name:ui("INSPECT"),exact:true}).click();await page.getByLabel(ui("Algorithm execution grid")).first().click({position:{x:5,y:5}});await expect(page.getByRole('heading',{name:/節點 \(/})).toBeVisible();await expect(page.locator('.node-details')).toContainText(ui("Parent"));
    await page.getByRole('button',{name:ui("ANALYZE"),exact:true}).click();await expect(page.getByRole('heading',{name:ui("Measured execution")})).toBeVisible();await expect(page.locator('table').first()).toContainText(ui("ARENA logical operations"));expect(errors).toEqual([]);
  });
}
test('4-way race, AI unavailability and export/import use saved logs',async({page})=>{
  await page.getByLabel(ui("Width"),{exact:true}).fill('16');await page.getByLabel(ui("Height"),{exact:true}).fill('16');await page.getByLabel(ui("Race format")).selectOption('4');await page.getByRole('button',{name:ui("Regenerate"),exact:false}).click();await expect(page.getByRole('button',{name:ui("Run experiment")})).toBeEnabled();await page.getByRole('button',{name:ui("Run experiment")}).click();await expect(page.getByLabel(ui("Event timeline"))).toBeVisible();await page.getByLabel(ui("End"),{exact:true}).click();await expect(page.getByLabel(ui("Algorithm execution grid"))).toHaveCount(4);
  const downloadPromise=page.waitForEvent('download');await page.getByRole('button',{name:ui("Export Run"),exact:false}).click();const download=await downloadPromise;const path=(await download.path())!;
  let executions=0;page.on('request',req=>{if(req.url().endsWith('/api/arena')&&req.postData()?.includes('"action":"race"'))executions++;});
  await page.locator('.file-actions input[type=file]').setInputFiles(path);await expect(page.getByLabel(ui("Event timeline"))).toHaveValue('0');await page.getByLabel(ui("End"),{exact:true}).click();expect(executions).toBe(0);
  await page.getByRole('button',{name:ui("ANALYZE"),exact:true}).click();await expect(page.getByText(ui('No AI interpretation service configured. The measured analysis above remains fully available.'), {exact:true})).toBeVisible();
});
test('desktop and narrow layout have no horizontal page overflow or runtime errors',async({page})=>{
  const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
  for(const width of [1536,390]){await page.setViewportSize({width,height:1024});await expect(page.getByRole('heading',{name:ui("Pathfinding arena")})).toBeVisible();expect(await page.evaluate(()=>document.documentElement.scrollWidth<=window.innerWidth)).toBe(true);}
  expect(errors).toEqual([]);
});
