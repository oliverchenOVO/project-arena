import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { spawn } from 'node:child_process';
import { resolve } from 'node:path';

export default defineConfig({
  plugins:[react(),{
    name:'arena-rust-dev-bridge',
    configureServer(server) {
      server.middlewares.use('/api/arena',(req,res)=>{
        if(req.method!=='POST') {res.statusCode=405;res.end();return;}
        let body='';let excessive=false;
        req.on('data',chunk=>{body+=chunk;if(body.length>128*1024*1024){excessive=true;req.destroy();}});
        req.on('end',()=>{
          if(excessive)return;
          const child=spawn(resolve('target/release',process.platform==='win32'?'arena.exe':'arena'),['bridge'],{stdio:['pipe','pipe','pipe'],windowsHide:true});
          let out='';let error='';
          child.stdout.on('data',chunk=>out+=chunk);
          child.stderr.on('data',chunk=>error+=chunk);
          child.on('error',err=>{if(!res.writableEnded){res.statusCode=503;res.end(JSON.stringify({error:err.message}));}});
          child.on('close',code=>{if(!res.writableEnded){res.setHeader('Content-Type','application/json');res.statusCode=code===0?200:400;res.end(code===0?out:JSON.stringify({error}));}});
          child.stdin.end(body);
        });
      });
    }
  }],
  server:{port:1431,strictPort:true},
  clearScreen:false
});
