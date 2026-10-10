/** JSON objects are unordered; arrays and numeric values retain their semantics. */
export function sameJson(a:unknown,b:unknown):boolean {const stable=(v:unknown)=>JSON.stringify(v,(_,value)=>value&&typeof value==='object'&&!Array.isArray(value)?Object.fromEntries(Object.keys(value).sort().map(k=>[k,value[k]])):value);return stable(a)===stable(b);}
