const {test}=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const vm=require('node:vm');
const path=require('node:path');
const html=fs.readFileSync(path.join(__dirname,'../assets/index.html'),'utf8');
const code=html.slice(html.indexOf('      function matchesProcess('),html.indexOf('      function renderCore('));
function harness() {
  const elements={coreSearch:{value:''},activityThreshold:{value:'0'},coreMatches:{},coreHint:{},coreProcesses:{innerHTML:'',matches:()=>false,contains:()=>false,querySelector:()=>null}};
  const context={document:{activeElement:null},data:{activity:{status:'running',lag_ms:100},processes:[]},$:(id)=>elements[id],pct:(v)=>v.toFixed(1),esc:(v)=>String(v)};
  vm.createContext(context);vm.runInContext(code,context);
  return {context,elements};
}
test('search combines with inclusive CPU/RAM thresholds and handles inaccessible affinity',()=>{
  const {context:c}=harness();
  const p={name:'Worker.EXE',pid:321,cpu:1,memory:100*1024*1024,affinity:[0]};
  assert.equal(c.matchesProcess(p,' worker ','busy',2),true);
  assert.equal(c.matchesProcess(p,'321','memory',2),true);
  assert.equal(c.matchesProcess(p,'missing','busy',2),false);
  assert.equal(c.matchesProcess({...p,cpu:.99},'','busy',2),false);
  assert.equal(c.matchesProcess({...p,memory:p.memory-1},'','memory',2),false);
  assert.equal(c.matchesProcess(p,'','restricted',2),true);
  for(const affinity of [null,[],[0,1]])assert.equal(c.matchesProcess({...p,affinity},'','restricted',2),false);
});
test('core threshold, search and idle are distinct; filters do not alter the source snapshot',()=>{
  const {context:c,elements:e}=harness();
  c.data.processes=[{pid:12,name:'worker',identity:'a',affinity:[0,1],allowed:[0,1]}];
  const sample={idle:94,unknown:0,rows:[{pid:12,usage:5,ms:50,history:[5]},{pid:99,usage:1,ms:10,history:[1]}]};
  e.activityThreshold.value='5';c.renderActivity({id:0},sample);
  assert.equal(e.coreMatches.textContent,'1 / 2');
  assert.match(e.coreProcesses.innerHTML,/worker/);assert.doesNotMatch(e.coreProcesses.innerHTML,/PID 99/);
  e.coreSearch.value='99';c.renderActivity({id:0},sample);
  assert.equal(e.coreMatches.textContent,'0 / 2');assert.match(e.coreProcesses.innerHTML,/Процессы не найдены/);
  assert.equal(sample.rows.length,2);
  e.activityThreshold.value='0';c.renderActivity({id:0},sample);
  assert.match(e.coreProcesses.innerHTML,/PID 99/);
});
test('focused activity row continues updating and keyboard focus is restored',()=>{
  const {context:c,elements:e}=harness();let restored=false;
  c.document.activeElement={dataset:{pid:'12'},matches:()=>false};
  e.coreProcesses.contains=()=>true;e.coreProcesses.querySelector=()=>({focus:()=>{restored=true}});
  c.data.processes=[{pid:12,name:'worker',identity:'a',affinity:[0],allowed:[0]}];
  c.renderActivity({id:0},{idle:90,unknown:0,rows:[{pid:12,usage:10,ms:100,history:[10]}]});
  assert.match(e.coreProcesses.innerHTML,/10.0%/);assert.equal(restored,true);
});
test('all referenced static UI IDs exist and inline JavaScript parses',()=>{
  const ids=new Set([...html.matchAll(/\bid="([^"]+)"/g)].map(m=>m[1]));
  for(const m of html.matchAll(/\$\("([^"]+)"\)/g))assert.ok(ids.has(m[1]),'Missing element '+m[1]);
  for(const m of html.matchAll(/<script[^>]*>([\s\S]*?)<\/script>/g))new vm.Script(m[1]);
});
test('Allowed and Denied remain independent of the global filter and retain process identity',()=>{
  const {context:c,elements:e}=harness();
  for(const id of ['coreTools','coreTitle','coreUsage','allowedTab','deniedTab','activityTab'])e[id]={classList:{toggle:()=>{}}};
  c.data.ready=true;c.data.cpus=[{id:0,usage:10}];c.filterCore=0;c.coreBusy=false;c.coreMode='allowed';c.bytes=String;
  c.data.processes=[{pid:12,name:'allowed',identity:'a',cpu:0,memory:0,affinity:[0],allowed:[0,1]},{pid:13,name:'denied',identity:'b',cpu:0,memory:0,affinity:[1],allowed:[0,1]}];
  vm.runInContext(html.slice(html.indexOf('      function renderCore('),html.indexOf('      function table(')),c);
  c.renderCore();assert.equal(e.coreMatches.textContent,'1 / 1');assert.match(e.coreProcesses.innerHTML,/data-identity="a"/);
  assert.equal(e.activityThreshold.hidden,true);
  c.coreMode='denied';c.renderCore();assert.match(e.coreProcesses.innerHTML,/data-identity="b"/);
  e.coreSearch.value='missing';c.renderCore();assert.equal(e.coreMatches.textContent,'0 / 1');
});
