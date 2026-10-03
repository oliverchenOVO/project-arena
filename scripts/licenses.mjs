import {execFileSync} from 'node:child_process';
import {readFileSync,readdirSync,existsSync,writeFileSync,mkdirSync} from 'node:fs';
import {join,dirname} from 'node:path';
const cargo=process.env.ARENA_CARGO||'cargo';
const metadata=JSON.parse(execFileSync(cargo,['metadata','--locked','--format-version','1','--manifest-path','src-tauri/Cargo.toml','--filter-platform','x86_64-pc-windows-msvc'],{maxBuffer:32*1024*1024,windowsHide:true}).toString());
const sections=['Project ARENA dependency license notices\n\nProject license: MIT. This file collects upstream license declarations and available license texts from locked dependencies.'];
for(const dependency of metadata.packages) {
  if(!dependency.source)continue;
  const directory=dirname(dependency.manifest_path);
  sections.push(`\n===== Rust: ${dependency.name} ${dependency.version} =====\nLicense: ${dependency.license||'see upstream license file'}\nSource: ${dependency.repository||dependency.source}`);
  const files=readdirSync(directory).filter(name=>/^(license|copying|notice)([-_.]|$)/i.test(name));
  for(const file of files) {try{sections.push(readFileSync(join(directory,file),'utf8'));}catch{/* A directory may hold license texts; declaration remains recorded. */}}
}
for(const name of ['react','react-dom','scheduler','@tauri-apps/api']) {
  const directory=join('node_modules',name);const pkg=JSON.parse(readFileSync(join(directory,'package.json'),'utf8'));
  sections.push(`\n===== Frontend: ${name} ${pkg.version} =====\nLicense: ${pkg.license}`);
  for(const file of readdirSync(directory).filter(name=>/^(license|copying|notice)([-_.]|$)/i.test(name))) {try{sections.push(readFileSync(join(directory,file),'utf8'));}catch{/* Directory, not a license text. */}}
}
mkdirSync('build-notices',{recursive:true});writeFileSync('build-notices/THIRD-PARTY-LICENSES.txt',sections.join('\n\n'));
if(!existsSync('LICENSE'))throw new Error('Project license missing');
console.log(`Collected ${metadata.packages.length} Rust package declarations and frontend license texts.`);
