import {describe,it,expect} from 'vitest';
import {readFileSync} from 'node:fs';
import {applyEvent,initialState,Replay,RacePlayback,playbackDelta,speeds} from '../playback';
import {clearGrid,editCell,validateMap} from '../editor';
import {defaultMap,metric,runConfig} from '../model';
import type {Grid,RunLog} from '../model';
import {summarize,UnavailableProvider} from '../explanation';

const log=JSON.parse(readFileSync(new URL('./fixtures/weighted-run.json',import.meta.url),'utf8')) as RunLog;
const grid:Grid={width:3,height:2,cells:['normal','normal','normal','normal','normal','normal'],start:0,goal:5};
describe('Rust event contract',()=>{
  it('event reducer terminal metrics equal actual Rust metrics',()=>{const state=initialState(log.grid.cells.length);log.events.forEach(e=>applyEvent(state,e));expect(state.metrics).toEqual(log.metrics);expect(state.completed).toBe(true);});
  it('checkpointed final replay equals prefix reduction',()=>{const replay=new Replay(log);replay.seek(log.events.length);const expected=initialState(log.grid.cells.length);log.events.forEach(e=>applyEvent(expected,e));expect(replay.state).toEqual(expected);});
  it('random forward and backward seeks equal every prefix',()=>{const replay=new Replay(log);for(let i=0;i<50;i++){const cursor=(i*137)%log.events.length;replay.seek(cursor);const expected=initialState(log.grid.cells.length);log.events.slice(0,cursor).forEach(e=>applyEvent(expected,e));expect(replay.state).toEqual(expected);}});
  it('checkpoint boundaries preserve state',()=>{const replay=new Replay(log);for(const cursor of [255,256,257,log.events.length]){if(cursor>log.events.length)continue;replay.seek(cursor);const state=initialState(log.grid.cells.length);log.events.slice(0,cursor).forEach(e=>applyEvent(state,e));expect(replay.state).toEqual(state);}});
  it('invalid seek never silently corrupts state',()=>{const replay=new Replay(log);expect(()=>replay.seek(-1)).toThrow();expect(()=>replay.seek(log.events.length+1)).toThrow();expect(()=>replay.seek(1.5)).toThrow();});
});
describe('shared playback clock',()=>{
  it('synchronizes contenders at the same cursor',()=>{const race=new RacePlayback([log,log]);race.seek(321);expect(race.replays[0].state).toEqual(race.replays[1].state);expect(race.replays.map(r=>r.cursor)).toEqual([321,321]);});
  it('pause prevents advances',()=>{const race=new RacePlayback([log]);race.update(100);expect(race.cursor).toBe(0);race.toggle();race.update(100);expect(race.cursor).toBe(100);race.pause();race.update(100);expect(race.cursor).toBe(100);});
  it('step event supports backward restoration',()=>{const race=new RacePlayback([log]);race.stepEvent(1);expect(race.cursor).toBe(1);race.stepEvent(-1);expect(race.cursor).toBe(0);expect(race.replays[0].state).toEqual(initialState(log.grid.cells.length));});
  it('step expansion stops on an actual expansion event',()=>{const race=new RacePlayback([log]);race.stepExpansion();expect(log.events[race.cursor-1].type).toBe('node_expanded');expect(race.replays[0].state.metrics.expanded).toBe(1);});
  it('beginning and end clamp the shared timeline',()=>{const race=new RacePlayback([log]);race.seek(999999);expect(race.cursor).toBe(log.events.length);race.seek(-1);expect(race.cursor).toBe(0);});
  it('MAX completes the saved log and stops',()=>{const race=new RacePlayback([log]);race.setSpeed('MAX');race.toggle();race.update(1);expect(race.cursor).toBe(log.events.length);expect(race.playing).toBe(false);});
  it.each(speeds.filter(s=>s!=='MAX'))('speed %s scales event rate',speed=>{expect(playbackDelta(100,speed)).toBe(100*Number(speed));});
  it('replay from end restarts without invoking a backend',()=>{const race=new RacePlayback([log]);race.seek(race.length);race.toggle();expect(race.cursor).toBe(0);expect(race.playing).toBe(true);});
});
describe('map editor validity',()=>{
  it('wall painting cannot overwrite start or goal',()=>{expect(editCell(grid,0,'wall')).toBe(grid);expect(editCell(grid,5,'wall')).toBe(grid);});
  it('placing endpoints on terrain makes them traversable',()=>{const wall=editCell(grid,2,'wall');expect(editCell(wall,2,'start').cells[2]).toBe('normal');expect(editCell(wall,2,'start').start).toBe(2);});
  it('supports weighted painting and erase without mutation',()=>{const updated=editCell(grid,1,'heavy');expect(updated.cells[1]).toBe('heavy');expect(grid.cells[1]).toBe('normal');expect(editCell(updated,1,'erase').cells[1]).toBe('normal');});
  it('allows start==goal as a valid zero-length problem',()=>{expect(editCell(grid,5,'start').start).toBe(grid.goal);});
  it('out of bounds painting is ignored',()=>{expect(editCell(grid,-1,'wall')).toBe(grid);expect(editCell(grid,99,'wall')).toBe(grid);});
  it('clear preserves endpoints',()=>{const painted=editCell(grid,2,'wall');expect(clearGrid(painted).cells).toEqual(grid.cells);expect(clearGrid(painted).goal).toBe(5);});
  it('invalid dimensions, unsafe seeds and densities are rejected',()=>{for(const config of [{...defaultMap,width:0},{...defaultMap,height:129},{...defaultMap,seed:Number.MAX_SAFE_INTEGER+1},{...defaultMap,wall_percent:101}])expect(()=>validateMap(config)).toThrow();expect(()=>validateMap(defaultMap)).not.toThrow();});
});
describe('analysis and metric presentation',()=>{
  it('formats unreachable cost as undefined rather than zero',()=>{expect(metric(null)).toBe('—');expect(metric(0)).toBe('0');expect(metric(1234)).toBe('1,234');});
  it('run configs never enable a heuristic for BFS/DFS/Dijkstra',()=>{expect(runConfig('bfs','manhattan').heuristic).toBe('zero');expect(runConfig('dfs','manhattan').heuristic).toBe('zero');expect(runConfig('greedy','manhattan').heuristic).toBe('manhattan');});
  it('structured summary separates facts, derived features and hypotheses',()=>{const summary=summarize([log],[]);expect(summary.measured).toHaveLength(1);expect(summary.derived).toHaveLength(1);expect(Array.isArray(summary.hypotheses)).toBe(true);});
  it('absent AI provider is independent of execution',async()=>{await expect(new UnavailableProvider().explain()).rejects.toThrow('unavailable');const replay=new Replay(log);replay.seek(log.events.length);expect(replay.state.completed).toBe(true);});
});
