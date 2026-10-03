import type { ArenaEvent, ReplayState, RunLog } from './model';

export function initialState(count:number):ReplayState {
  return {nodes:Array.from({length:count},()=>({discovered:false,expanded:false,frontier_entries:0,g:null,h:null,f:null,parent:null,expanded_at:null})),current:null,path:[],completed:false,found:false,frontier:0,finiteCosts:0,parents:0,metrics:{expanded:0,generated:0,edges_evaluated:0,frontier_pushes:0,frontier_pops:0,cost_updates:0,relaxation_comparisons:0,peak_frontier:0,peak_discovered:0,peak_state_entries:0,path_length:null,path_cost:null,logical_ops:0}};
}
/** The reducer knows the event schema, never the algorithm implementation. */
export function applyEvent(s:ReplayState,e:ArenaEvent):void {
  const m=s.metrics;
  switch(e.type) {
    case 'node_discovered':s.nodes[e.node].discovered=true;m.generated++;m.peak_discovered=m.generated;break;
    case 'node_expanded':s.nodes[e.node].expanded=true;s.nodes[e.node].expanded_at=e.sequence;s.current=e.node;m.expanded++;m.logical_ops++;break;
    case 'frontier_added':s.nodes[e.node].frontier_entries++;s.frontier++;m.frontier_pushes++;m.logical_ops++;m.peak_frontier=Math.max(m.peak_frontier,s.frontier);break;
    case 'frontier_removed':s.nodes[e.node].frontier_entries--;s.frontier--;m.frontier_pops++;m.logical_ops++;break;
    case 'edge_evaluated':m.edges_evaluated++;m.logical_ops++;break;
    case 'relaxation_compared':m.relaxation_comparisons++;m.logical_ops++;break;
    case 'cost_updated':{const n=s.nodes[e.node];if(n.g===null)s.finiteCosts++;n.g=e.g;n.h=e.h;n.f=e.f;m.cost_updates++;break;}
    case 'parent_updated':if(s.nodes[e.node].parent===null)s.parents++;s.nodes[e.node].parent=e.parent;break;
    case 'path_reconstructed':s.path=[...e.path];m.path_length=e.path.length-1;m.path_cost=e.cost;break;
    case 'run_completed':s.completed=true;s.found=e.found;break;
  }
  m.peak_state_entries=Math.max(m.peak_state_entries,m.generated+s.finiteCosts+s.parents+s.frontier);
}

export class Replay {
  readonly checkpoints:ReplayState[]=[];
  readonly interval:number;
  state:ReplayState;
  cursor=0;
  constructor(readonly log:RunLog) {
    this.interval=Math.max(256,Math.ceil(log.events.length/32));
    this.state=initialState(log.grid.cells.length);
    this.checkpoints.push(structuredClone(this.state));
    for(let i=0;i<log.events.length;i++) {applyEvent(this.state,log.events[i]);if((i+1)%this.interval===0)this.checkpoints.push(structuredClone(this.state));}
    this.state=initialState(log.grid.cells.length);
  }
  seek(target:number):void {
    if(!Number.isInteger(target)||target<0||target>this.log.events.length)throw new Error('Invalid seek cursor');
    if(target>=this.cursor && target-this.cursor<=this.interval) {this.advance(target);return;}
    const checkpoint=Math.floor(target/this.interval);
    this.state=structuredClone(this.checkpoints[checkpoint]);this.cursor=checkpoint*this.interval;this.advance(target);
  }
  advance(target:number):void {while(this.cursor<target)applyEvent(this.state,this.log.events[this.cursor++]);}
}
export const speeds=[0.25,0.5,1,2,4,8,'MAX'] as const;
export type Speed=typeof speeds[number];
export function playbackDelta(elapsedMs:number,speed:Speed):number {return speed==='MAX'?Infinity:Math.max(0,elapsedMs)*speed;}
/** All contenders share one event-count clock. One event is one schema event. */
export class RacePlayback {
  readonly replays:Replay[];
  readonly length:number;
  cursor=0;
  playing=false;
  speed:Speed=1;
  private fraction=0;
  private revision=0;
  private listeners=new Set<()=>void>();
  constructor(readonly logs:RunLog[]) {this.replays=logs.map(log=>new Replay(log));this.length=Math.max(0,...logs.map(log=>log.events.length));}
  subscribe=(listener:()=>void)=>{this.listeners.add(listener);return ()=>{this.listeners.delete(listener);};};
  snapshot=()=>this.revision;
  private publish(){this.revision++;this.listeners.forEach(listener=>listener());}
  seek(target:number){const next=Math.max(0,Math.min(this.length,Math.floor(target)));this.replays.forEach(replay=>replay.seek(Math.min(next,replay.log.events.length)));this.cursor=next;if(next===this.length)this.playing=false;this.fraction=0;this.publish();}
  toggle(){if(this.cursor===this.length)this.seek(0);this.playing=!this.playing;this.publish();}
  pause(){this.playing=false;this.publish();}
  setSpeed(speed:Speed){this.speed=speed;this.publish();}
  stepEvent(direction:1|-1){this.playing=false;this.seek(this.cursor+direction);}
  stepExpansion(){this.playing=false;let target=this.length;for(const log of this.logs){const next=log.events.findIndex((event,index)=>index>=this.cursor&&event.type==='node_expanded');if(next>=0)target=Math.min(target,next+1);}this.seek(target);}
  update(elapsed:number){if(!this.playing)return;const amount=playbackDelta(elapsed,this.speed)+this.fraction;const whole=Math.floor(amount);if(whole>0){const remainder=amount-whole;this.seek(this.cursor+whole);this.fraction=Number.isFinite(remainder)?remainder:0;}else this.fraction=amount;}
}
