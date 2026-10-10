import {useState} from 'react';
import App from './App';
import DomainArena from './components/DomainArena';
import type {ArenaDomain} from './domains';
export default function Arena(){const [domain,setDomain]=useState<ArenaDomain>('pathfinding');return <><div hidden={domain!=='pathfinding'}><App active={domain==='pathfinding'} onDomain={setDomain}/></div>{(['sorting','graph'] as const).map(d=><div key={d} hidden={domain!==d}><DomainArena active={domain===d} domain={d} onDomain={setDomain}/></div>)}</>;}
