import {defineConfig} from '@playwright/test';
const port=Number(process.env.ARENA_TEST_PORT??1431);
if(!Number.isInteger(port)||port<1||port>65535)throw new Error('Invalid ARENA_TEST_PORT');
const url=`http://127.0.0.1:${port}`;
export default defineConfig({testDir:'./tests/e2e',timeout:90000,expect:{timeout:15000},fullyParallel:false,workers:1,reporter:[['list'],['json',{outputFile:'test-results/e2e-report.json'}]],use:{baseURL:url,viewport:{width:1536,height:1024},headless:true,launchOptions:{args:['--no-proxy-server']}},webServer:{command:`npm run dev -- --port ${port}`,url,reuseExistingServer:true,timeout:120000}});
