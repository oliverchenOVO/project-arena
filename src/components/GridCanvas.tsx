import { useEffect,useRef } from 'react';
import type {Grid,ReplayState} from '../model';
import type {RacePlayback} from '../playback';

interface Props {grid:Grid;accent?:string;store:RacePlayback|null;index?:number;onCell:(node:number)=>void;editable?:boolean;selected:number|null}
export default function GridCanvas({grid,accent='#4dd6c6',store,index=0,onCell,editable=false,selected}:Props) {
  const ref=useRef<HTMLCanvasElement>(null);
  const dragging=useRef(false);
  const lastNode=useRef(-1);
  useEffect(()=>{
    const canvas=ref.current!;const context=canvas.getContext('2d')!;
    const draw=()=> {
      const rect=canvas.getBoundingClientRect();const ratio=window.devicePixelRatio||1;
      if(canvas.width!==Math.round(rect.width*ratio)||canvas.height!==Math.round(rect.height*ratio)) {canvas.width=Math.round(rect.width*ratio);canvas.height=Math.round(rect.height*ratio);}
      context.setTransform(ratio,0,0,ratio,0,0);
      const cw=rect.width/grid.width,ch=rect.height/grid.height;
      const state:ReplayState|undefined=store?.replays[index]?.state;
      const path=new Set(state?.path);
      context.fillStyle='#0c121a';context.fillRect(0,0,rect.width,rect.height);
      for(let i=0;i<grid.cells.length;i++) {
        const x=(i%grid.width)*cw,y=Math.floor(i/grid.width)*ch,n=state?.nodes[i];
        const terrain=grid.cells[i];
        let fill=terrain==='wall'?'#454f5e':terrain==='mud'?'#705d42':terrain==='water'?'#24547b':terrain==='heavy'?'#77503c':'#111b27';
        if(n?.discovered)fill=accent+'25';
        if(n?.expanded)fill=accent+'80';
        if(path.has(i))fill='#d1deed';
        if(state?.current===i)fill='#e8bd65';
        if(i===grid.start)fill='#74dcc0';if(i===grid.goal)fill='#ed8d82';
        const gap=cw>5?1:0.3;
        context.fillStyle=fill;context.fillRect(x+gap,y+gap,Math.max(1,cw-gap*2),Math.max(1,ch-gap*2));
        if(n?.frontier_entries&&!n.expanded&&!path.has(i)&&i!==grid.start&&i!==grid.goal) {context.strokeStyle=accent;context.lineWidth=1;context.strokeRect(x+1.5,y+1.5,cw-3,ch-3);}
        if((i===grid.start||i===grid.goal)&&cw>=10) {context.fillStyle='#0b0e13';context.font=`bold ${Math.max(9,Math.min(14,cw*.65))}px Consolas`;context.textAlign='center';context.textBaseline='middle';context.fillText(i===grid.start?(grid.start===grid.goal?'S/G':'S'):'G',x+cw/2,y+ch/2);}
        if(i===selected) {context.strokeStyle='#ffffff';context.lineWidth=2;context.strokeRect(x+1,y+1,cw-2,ch-2);}
      }
    };
    const observer=new ResizeObserver(draw);observer.observe(canvas);
    const unsubscribe=store?.subscribe(draw);draw();return ()=>{observer.disconnect();unsubscribe?.();};
  },[grid,accent,store,index,selected]);
  const cellAt=(event:React.PointerEvent<HTMLCanvasElement>)=>{const rect=event.currentTarget.getBoundingClientRect();const x=Math.min(grid.width-1,Math.max(0,Math.floor((event.clientX-rect.left)/rect.width*grid.width)));const y=Math.min(grid.height-1,Math.max(0,Math.floor((event.clientY-rect.top)/rect.height*grid.height)));return y*grid.width+x;};
  const paint=(event:React.PointerEvent<HTMLCanvasElement>)=>{const node=cellAt(event);if(lastNode.current!==node){lastNode.current=node;onCell(node);}};
  return <canvas ref={ref} className={'grid-canvas'+(editable?' editable':'')} style={{aspectRatio:`${grid.width}/${grid.height}`}} aria-label={editable?'Editable problem grid':'Algorithm execution grid'}
    onPointerDown={event=>{if(event.button!==0)return;dragging.current=true;lastNode.current=-1;event.currentTarget.setPointerCapture(event.pointerId);paint(event);}}
    onPointerMove={event=>{if(dragging.current&&editable)paint(event);}}
    onPointerUp={()=>{dragging.current=false;lastNode.current=-1;}} onPointerCancel={()=>{dragging.current=false;lastNode.current=-1;}}/>;
}
