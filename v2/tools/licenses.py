"""Collect exact locked Rust and installed validation-tool license evidence."""
from pathlib import Path
import subprocess,json,hashlib
ROOT=Path(__file__).resolve().parent.parent
OUT=ROOT/'licenses/dependencies';OUT.mkdir(parents=True,exist_ok=True)
records={}
def collect(name,version,license_name,base,ecosystem,source):
    key=(ecosystem,name,version)
    if key in records:return
    dest=OUT/ecosystem/(name.replace('/','__')+'-'+version);dest.mkdir(parents=True,exist_ok=True)
    files=[]
    for p in sorted(base.iterdir()):
        if p.is_file() and p.name.upper().startswith(('LICENSE','LICENCE','COPYING','COPYRIGHT','NOTICE','UNLICENSE')):
            data=p.read_bytes();(dest/p.name).write_bytes(data);files.append(dict(path=str((dest/p.name).relative_to(ROOT)),sha256=hashlib.sha256(data).hexdigest(),bytes=len(data)))
    if not files and name.startswith('@esbuild/'):
        for store in [ROOT/'tools/node_modules/.pnpm', ROOT/'.build/v1/node_modules/.pnpm']:
            parent=store/('esbuild@'+version)/'node_modules/esbuild'
            if parent.is_dir():
                for p in parent.glob('LICENSE*'):
                    data=p.read_bytes(); (dest/p.name).write_bytes(data); files.append(dict(path=str((dest/p.name).relative_to(ROOT)),sha256=hashlib.sha256(data).hexdigest(),bytes=len(data),noticeSource='esbuild@'+version))
                break
    if not files:
        # A few type-only tools put their MIT grant in README instead.
        for p in sorted(base.glob('*')):
            if p.is_file() and p.name.lower().startswith('readme') and 'license' in p.read_text(errors='replace').lower():
                data=p.read_bytes();(dest/p.name).write_bytes(data);files.append(dict(path=str((dest/p.name).relative_to(ROOT)),sha256=hashlib.sha256(data).hexdigest(),bytes=len(data)))
    if not files and name=='@raycast/api':
        p=ROOT/'fixtures/upstream/LICENSE';data=p.read_bytes();(dest/'LICENSE').write_bytes(data);files.append(dict(path=str((dest/'LICENSE').relative_to(ROOT)),sha256=hashlib.sha256(data).hexdigest(),bytes=len(data),noticeSource='https://github.com/raycast/extensions/blob/6734d85992290066fd31a7f5bee3cca3d3545a1a/LICENSE'))
    if not files:
        override=ROOT/'licenses/overrides'/(name.replace('/','__')+'-'+version)
        if override.is_dir():
            for p in override.glob('LICENSE*'):
                data=p.read_bytes();(dest/p.name).write_bytes(data);files.append(dict(path=str((dest/p.name).relative_to(ROOT)),sha256=hashlib.sha256(data).hexdigest(),bytes=len(data),noticeSource=json.loads((override/'evidence.json').read_text())['url']))
    if not files:raise RuntimeError('Missing license evidence '+repr(key))
    records[key]=dict(ecosystem=ecosystem,name=name,version=version,license=license_name,source=source,files=files)
for manifest in [ROOT/'Cargo.toml',ROOT/'.build/beta-harness/Cargo.toml']:
    metadata=json.loads(subprocess.check_output(['cargo','metadata','--locked','--format-version','1','--manifest-path',str(manifest)],text=True))
    for pkg in metadata['packages']:
        if pkg['source'] and pkg['source'].startswith('registry+'):
            collect(pkg['name'],pkg['version'],pkg['license'],Path(pkg['manifest_path']).parent,'rust',pkg['source'])
for folder in [ROOT/'tools/node_modules/.pnpm',ROOT/'.build/v1/node_modules/.pnpm']:
    for dep in folder.iterdir():
        nm=dep/'node_modules'
        if not nm.is_dir():continue
        for p in list(nm.glob('*/package.json'))+list(nm.glob('@*/*/package.json')):
            pkg=json.loads(p.read_text());lic=pkg.get('license') or pkg.get('licenses')
            collect(pkg['name'],pkg['version'],lic,p.parent,'npm','https://www.npmjs.com/package/'+pkg['name']+'/v/'+pkg['version'])
result=sorted(records.values(),key=lambda r:(r['ecosystem'],r['name'],r['version']))
(ROOT/'licenses/dependencies.json').write_text(json.dumps(result,indent=2)+'\n')
print('License evidence:',len(result),'packages; expressions:',sorted({str(r['license']) for r in result}))
