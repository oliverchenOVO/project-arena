import {readFileSync} from 'node:fs';
const en=JSON.parse(readFileSync('src/i18n/en.json','utf8')) as Record<string,string>;
const zh=JSON.parse(readFileSync('src/i18n/zh-TW.json','utf8')) as Record<string,string>;
export const ui=(key:string,locale:'zh-TW'|'en'='zh-TW',values:Record<string,string|number>={})=>(locale==='en'?en:zh)[key]?.replace(/\{(\w+)\}/g,(m,k)=>String(values[k]??m))??key;
