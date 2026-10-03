import type {Grid,MapConfig,Terrain} from './model';
export type Tool=Terrain|'start'|'goal'|'erase';
export function validateMap(config:MapConfig):void {
  if(!Number.isSafeInteger(config.seed)||config.seed<0)throw new Error('Seed must be a safe non-negative integer');
  if(!Number.isInteger(config.width)||!Number.isInteger(config.height)||config.width<1||config.height<1||config.width>128||config.height>128)throw new Error('Dimensions must be integers from 1 to 128');
  if(!Number.isInteger(config.wall_percent)||config.wall_percent<0||config.wall_percent>100)throw new Error('Obstacle density must be 0–100');
}
export function editCell(grid:Grid,node:number,tool:Tool):Grid {
  if(!Number.isInteger(node)||node<0||node>=grid.cells.length)return grid;
  const cells=[...grid.cells];
  if(tool==='start'||tool==='goal') {cells[node]='normal';return {...grid,cells,[tool]:node};}
  if(node===grid.start||node===grid.goal)return grid;
  cells[node]=tool==='erase'?'normal':tool;return {...grid,cells};
}
export function clearGrid(grid:Grid):Grid {return {...grid,cells:grid.cells.map(()=>'normal')};}
