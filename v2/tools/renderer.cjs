// Render the unchanged beta components and exercise their real parsers,
// action selection and form reconciliation. All OS/IPC calls are forbidden.
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const Module = require('node:module');
const { JSDOM } = require('jsdom');
const React = require('react');
const TestRenderer = require('react-test-renderer');
const esbuild = require('esbuild');
const root = path.resolve(__dirname, '..');
const dom = new JSDOM('<!doctype html><html><body></body></html>', { url: 'https://fixture.invalid' });
for (const key of ['window','document','navigator','DOMParser','Element','HTMLElement','CustomEvent']) {
  Object.defineProperty(globalThis,key,{value:key==='window'?dom.window:dom.window[key],configurable:true});
}
window.tezbar = new Proxy({}, { get(_target, key) { return () => { throw Error('Forbidden OS/IPC call: '+String(key)); }; } });
function compile(entry, tail='') {
  const result = esbuild.buildSync({ stdin: { contents:entry+tail,resolveDir:root,loader:'tsx' },bundle:true,platform:'node',format:'cjs',jsx:'automatic',write:false,nodePaths:[path.join(__dirname,'node_modules')],external:['react','react-dom','react/jsx-runtime'] });
  const mod = new Module(path.join(__dirname,'fixture.cjs'),module);mod.paths=module.paths;mod.filename=path.join(__dirname,'fixture.cjs');mod._compile(result.outputFiles[0].text,mod.filename);return mod.exports;
}
const { ExtensionRuntimeSurface: Surface } = compile("export {ExtensionRuntimeSurface} from './fixtures/beta/src/renderer/src/raycast-api/index.tsx';");
const { reconcileFormValues } = compile("export {reconcileFormValues} from './fixtures/beta/src/renderer/src/raycast-api/form-runtime.tsx';");
const views = JSON.parse(fs.readFileSync(path.join(root,'.build/views.json'),'utf8'));
const oracle = JSON.parse(fs.readFileSync(path.join(root,'tests/oracle.json'),'utf8'));
const invocations=[];const searches=[];let outerBack=0;let renderer;
function props(view){return {sessionId:'fixture',extensionId:'com.tezbar.native.markdown-reference',commandName:'markdown-reference',...view,onBack:()=>outerBack++,onSearchTextChanged:q=>searches.push(q),onLoadMore:()=>{throw Error('No pagination');},onInvokeAction:(id,values)=>invocations.push({id,values})};}
function mount(view){TestRenderer.act(()=>{if(renderer)renderer.update(React.createElement(Surface,props(view)));else renderer=TestRenderer.create(React.createElement(Surface,props(view)));});}
function buttons(){return renderer.root.findAll(n=>n.type==='button'&&n.props['data-idx']!==undefined);}
mount(views.list);assert.equal(buttons().length,15);
for(let i=0;i<15;i++){TestRenderer.act(()=>buttons()[i].props.onClick());assert.equal(invocations.at(-1).id,views.list.actions[i].id);}
mount(views.search);assert.equal(buttons().length,5);assert.equal(renderer.root.findByType('input').props.value,'LI');
TestRenderer.act(()=>buttons()[0].props.onClick());assert.equal(invocations.at(-1).id,views.search.actions[0].id);
const key=(name,extra={})=>({key:name,metaKey:false,ctrlKey:false,altKey:false,shiftKey:false,repeat:false,preventDefault(){},stopPropagation(){},...extra});
TestRenderer.act(()=>buttons()[0].props.onMouseEnter());
const listKeys=()=>renderer.root.findAll(n=>n.type==='div'&&typeof n.props.onKeyDown==='function')[0];
TestRenderer.act(()=>listKeys().props.onKeyDown(key('ArrowDown')));
TestRenderer.act(()=>listKeys().props.onKeyDown(key('Enter')));assert.equal(invocations.at(-1).id,views.search.actions[1].id);
TestRenderer.act(()=>listKeys().props.onKeyDown(key('k',{metaKey:true})));
const overlay=renderer.root.findAll(n=>n.type==='button'&&n.props.children==='View');assert.equal(overlay.length,1);
TestRenderer.act(()=>overlay[0].props.onClick());assert.equal(invocations.at(-1).id,views.search.actions[1].id);
mount(views.detail);const back=renderer.root.findAll(n=>n.type==='button'&&n.props.children==='Back')[0];TestRenderer.act(()=>back.props.onClick());assert.equal(invocations.at(-1).id,views.detail.root.props.backActionId);assert.equal(outerBack,0);
mount(views.back);assert.equal(renderer.root.findByType('input').props.value,'LI');assert.equal(buttons().length,5);
for(let i=0;i<views.topics.length;i++){
  const view=views.topics[i];mount(view);assert.equal(view.root.props.markdown,oracle.markdowns[i]);
  const markdown=renderer.root.findAll(n=>n.type==='div'&&n.props.dangerouslySetInnerHTML)[0];assert.ok(markdown);
  const doc=new JSDOM(markdown.props.dangerouslySetInnerHTML.__html).window.document;
  assert.equal(doc.querySelector('h1').textContent,oracle.references[i].name);
  assert.ok(doc.body.textContent.includes('Examples'));
  assert.equal(doc.querySelectorAll('pre').length,0,'No generic code-block clipboard UI is introduced by this source');
  assert.equal(doc.querySelectorAll('script,img,iframe').length,0,'Examples stay literal; no remote media loads');
  for(const a of doc.querySelectorAll('a[href]'))assert.match(a.getAttribute('href'),/^https?:\/\//);
}
// Real beta submit-only form reconciliation is checked, even though this
// source declares no forms, fields, preferences or storage.
const fields=[{key:'draft:Form.TextField',id:'draft',type:'Form.TextField',title:'Draft',value:''}];
assert.deepEqual(reconcileFormValues(fields,{draft:'unsubmitted'},structuredClone(fields)),{draft:'unsubmitted'});
TestRenderer.act(()=>renderer.unmount());
console.log('Beta renderer: all list rows/actions, 15 parsed/rendered details, server search, keyboard handlers and row action overlay, Back/draft restoration, real form reconciliation; no OS/IPC');
