import {describe,it,expect} from 'vitest';
import {readFileSync,readdirSync} from 'node:fs';
import ts from 'typescript';
import {dictionaries,resolveLocale,translate} from '../i18n';
import {analysisHeadline} from '../components/Showcase';
import type {RunLog} from '../model';
const log=JSON.parse(readFileSync(new URL('./fixtures/weighted-run.json',import.meta.url),'utf8')) as RunLog;
describe('localization contracts',()=>{
 it('both dictionaries contain exactly the same nonempty keys',()=>{expect(Object.keys(dictionaries.en).sort()).toEqual(Object.keys(dictionaries['zh-TW']).sort());for(const d of Object.values(dictionaries))for(const value of Object.values(d))expect(value.trim().length).toBeGreaterThan(0);});
 it('every literal translation call exists in both dictionaries',()=>{const files=['src/App.tsx',...readdirSync('src/components').filter(f=>f.endsWith('.tsx')).map(f=>'src/components/'+f)];for(const file of files){const ast=ts.createSourceFile(file,readFileSync(file,'utf8'),ts.ScriptTarget.Latest,true,ts.ScriptKind.TSX);function visit(n:ts.Node){if(ts.isCallExpression(n)&&n.expression.getText(ast)==='t'&&n.arguments[0]&&ts.isStringLiteral(n.arguments[0]))expect(dictionaries.en).toHaveProperty(n.arguments[0].text);ts.forEachChild(n,visit);}visit(ast);}});
 it('uses stored preference before browser language and Traditional Chinese fallback',()=>{expect(resolveLocale('en',['zh-TW'])).toBe('en');expect(resolveLocale('zh-TW',['en-US'])).toBe('zh-TW');expect(resolveLocale(null,['zh-TW','en-US'])).toBe('zh-TW');expect(resolveLocale(null,['en-US'])).toBe('en');expect(resolveLocale('invalid',['fr'])).toBe('zh-TW');});
 it('falls back to English for a missing translation then to the key',()=>{expect(translate('zh-TW','Expanded',{},{})).toBe('Expanded');expect(translate('en','unknown-key')).toBe('unknown-key');});
 it('interpolates counts without altering canonical data',()=>{const before=JSON.stringify(log);expect(translate('zh-TW','In this run, {algorithm} expanded {expanded} nodes with path cost {cost}.',{algorithm:'Dijkstra',expanded:log.metrics.expanded,cost:log.metrics.path_cost??'—'})).toContain(String(log.metrics.expanded));expect(JSON.stringify(log)).toBe(before);});
 it('headline requires equal reachable cost and computes observed expansion reduction',()=>{const d={...log,config:{...log.config,algorithm:'dijkstra' as const},metrics:{...log.metrics,expanded:100,path_cost:20}};const a={...d,config:{...d.config,algorithm:'astar' as const},metrics:{...d.metrics,expanded:33}};expect(analysisHeadline([a,d]).values.reduction).toBe('67.0');expect(analysisHeadline([{...a,metrics:{...a.metrics,path_cost:21}},d]).values).toEqual({});expect(analysisHeadline([{...a,metrics:{...a.metrics,path_cost:null}},d]).values).toEqual({});});
});
