import type {RacePlayback} from './playback';

/** Ignore native controls so typing, button activation and slider arrows keep their normal behavior. */
export function handleReplayShortcut(event:KeyboardEvent,store:Pick<RacePlayback,'toggle'|'stepEvent'|'stepExpansion'|'pause'|'seek'|'length'>):boolean {
  if(event.defaultPrevented||event.repeat||event.isComposing||event.ctrlKey||event.metaKey||event.altKey)return false;
  if(event.target instanceof Element&&event.target.closest('input,textarea,select,button,a,summary,[contenteditable]:not([contenteditable="false"]),[role="slider"],[role="textbox"]'))return false;
  if(event.key===' '){store.toggle();}
  else if(event.key==='ArrowLeft'&&!event.shiftKey){store.stepEvent(-1);}
  else if(event.key==='ArrowRight'){if(event.shiftKey)store.stepExpansion();else store.stepEvent(1);}
  else if(event.key==='Home'&&!event.shiftKey){store.pause();store.seek(0);}
  else if(event.key==='End'&&!event.shiftKey){store.pause();store.seek(store.length);}
  else return false;
  event.preventDefault();return true;
}
