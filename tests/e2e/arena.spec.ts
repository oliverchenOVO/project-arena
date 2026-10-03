import {test,expect} from '@playwright/test';

test.beforeEach(async({page})=>{await page.goto('/');await expect(page.getByRole('button',{name:'Run experiment'})).toBeEnabled({timeout:60000});});
test('identity, editor validity, drag painting and clear',async({page})=>{
  await expect(page).toHaveTitle(/Project ARENA/);await expect(page.getByRole('heading',{name:'Pathfinding arena'})).toBeVisible();
  await page.getByRole('button',{name:'BUILD',exact:true}).click();
  const canvas=page.getByLabel('Editable problem grid');const box=(await canvas.boundingBox())!;
  await page.getByRole('button',{name:'heavy',exact:false}).click();
  await page.mouse.move(box.x+box.width*.2,box.y+box.height*.2);await page.mouse.down();await page.mouse.move(box.x+box.width*.5,box.y+box.height*.2,{steps:20});await page.mouse.up();
  await page.getByRole('button',{name:'Clear',exact:true}).click();
  await page.getByLabel('Width',{exact:true}).fill('0');await page.getByRole('button',{name:'Regenerate',exact:false}).click();await expect(page.getByRole('alert')).toContainText('Dimensions');
  await page.getByLabel('Width',{exact:true}).fill('16');await page.getByLabel('Height',{exact:true}).fill('16');await page.getByRole('button',{name:'Regenerate',exact:false}).click();await expect(page.getByRole('alert')).toHaveCount(0);
});
for(const [a,b] of [['bfs','dfs'],['dijkstra','astar'],['astar','greedy']])for(const family of ['open','backtracker','weighted','corridors','obstacles']) {
  test(`${a} vs ${b} / ${family}: race, replay, inspect and analysis`,async({page})=>{
    const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
    await page.getByLabel('Width',{exact:true}).fill('16');await page.getByLabel('Height',{exact:true}).fill('16');await page.getByLabel('Map family').selectOption(family);await page.getByLabel('Algorithm A',{exact:true}).selectOption(a);await page.getByLabel('Algorithm B',{exact:true}).selectOption(b);await page.getByRole('button',{name:'Regenerate',exact:false}).click();
    await expect(page.getByRole('button',{name:'Run experiment'})).toBeEnabled();await page.getByRole('button',{name:'Run experiment'}).click();
    await expect(page.getByLabel('Event timeline')).toBeVisible();await page.getByLabel('End',{exact:true}).click();const finalText=await page.locator('.race-metrics').allTextContents();
    await page.getByLabel('Beginning',{exact:true}).click();await page.getByLabel('Step expansion',{exact:true}).click();await page.getByLabel('Step backward',{exact:true}).click();await page.getByLabel('Step event',{exact:true}).click();await page.getByLabel('Playback speed').selectOption('8');await page.getByLabel('End',{exact:true}).click();expect(await page.locator('.race-metrics').allTextContents()).toEqual(finalText);
    await page.getByRole('button',{name:'INSPECT',exact:true}).click();await page.getByLabel('Algorithm execution grid').first().click({position:{x:5,y:5}});await expect(page.getByRole('heading',{name:/Node \(/})).toBeVisible();await expect(page.locator('.node-details')).toContainText('Parent');
    await page.getByRole('button',{name:'ANALYZE',exact:true}).click();await expect(page.getByRole('heading',{name:'Measured execution'})).toBeVisible();await expect(page.locator('table')).toContainText('ARENA logical operations');expect(errors).toEqual([]);
  });
}
test('4-way race, AI unavailability and export/import use saved logs',async({page})=>{
  await page.getByLabel('Width',{exact:true}).fill('16');await page.getByLabel('Height',{exact:true}).fill('16');await page.getByLabel('Race format').selectOption('4');await page.getByRole('button',{name:'Regenerate',exact:false}).click();await expect(page.getByRole('button',{name:'Run experiment'})).toBeEnabled();await page.getByRole('button',{name:'Run experiment'}).click();await expect(page.getByLabel('Event timeline')).toBeVisible();await page.getByLabel('End',{exact:true}).click();await expect(page.getByLabel('Algorithm execution grid')).toHaveCount(4);
  const downloadPromise=page.waitForEvent('download');await page.getByRole('button',{name:'Export Run',exact:false}).click();const download=await downloadPromise;const path=(await download.path())!;
  let executions=0;page.on('request',req=>{if(req.url().endsWith('/api/arena')&&req.postData()?.includes('"action":"race"'))executions++;});
  await page.locator('input[type=file]').setInputFiles(path);await expect(page.getByLabel('Event timeline')).toHaveValue('0');await page.getByLabel('End',{exact:true}).click();expect(executions).toBe(0);
  await page.getByRole('button',{name:'ANALYZE',exact:true}).click();await page.getByRole('button',{name:'Check AI provider'}).click();await expect(page.getByText('AI explanation unavailable: no provider configured', {exact:true})).toBeVisible();
});
test('desktop and narrow layout have no horizontal page overflow or runtime errors',async({page})=>{
  const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
  for(const width of [1536,390]){await page.setViewportSize({width,height:1024});await expect(page.getByRole('heading',{name:'Pathfinding arena'})).toBeVisible();expect(await page.evaluate(()=>document.documentElement.scrollWidth<=window.innerWidth)).toBe(true);}
  expect(errors).toEqual([]);
});
