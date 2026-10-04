import {createContext,useContext,useEffect,useMemo,useState} from 'react';
import type {ReactNode} from 'react';
import en from './en.json';
import zh from './zh-TW.json';
export type Locale='zh-TW'|'en';
export type MessageKey=keyof typeof en;
export const dictionaries={en,'zh-TW':zh};
export const localeStorageKey='arena.locale.v1';
export function resolveLocale(stored:string|null,languages:readonly string[]):Locale {
 if(stored==='en'||stored==='zh-TW')return stored;
 for(const language of languages){if(/^zh/i.test(language))return 'zh-TW';if(/^en/i.test(language))return 'en';}
 return 'zh-TW';
}
export function translate(locale:Locale,key:string,values:Record<string,string|number>={},dictionary:Record<string,string>=dictionaries[locale]):string {
 const message=dictionary[key]??(en as Record<string,string>)[key]??key;
 return message.replace(/\{(\w+)\}/g,(match,name)=>String(values[name]??match));
}
function initialLocale():Locale {
 let stored:string|null=null;try{stored=localStorage.getItem(localeStorageKey);}catch{/* Browser language still applies when storage is unavailable. */}
 return resolveLocale(stored,typeof navigator==='undefined'?[]:[...navigator.languages,navigator.language]);
}
const Context=createContext({locale:'zh-TW' as Locale,setLocale:(locale:Locale)=>{void locale;},t:(key:string,values?:Record<string,string|number>)=>translate('zh-TW',key,values)});
export function LocaleProvider({children}:{children:ReactNode}){
 const[locale,setLocale]=useState<Locale>(initialLocale);
 useEffect(()=>{document.documentElement.lang=locale;try{localStorage.setItem(localeStorageKey,locale);}catch{/* Read-only storage still permits an in-memory preference. */}},[locale]);
 const value=useMemo(()=>({locale,setLocale,t:(key:string,values?:Record<string,string|number>)=>translate(locale,key,values)}),[locale]);
 return <Context.Provider value={value}>{children}</Context.Provider>;
}
export const useLocale=()=>useContext(Context);
export function LanguageSwitch(){const{locale,setLocale,t}=useLocale();return <div className="language-switch" role="group" aria-label={t('Language')}><button aria-pressed={locale==='zh-TW'} onClick={()=>setLocale('zh-TW')}>繁中</button><span aria-hidden="true">|</span><button aria-pressed={locale==='en'} onClick={()=>setLocale('en')}>EN</button></div>;}
