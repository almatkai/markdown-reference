"""Package and re-run tests using only the extracted package's source files."""
from pathlib import Path
import subprocess,tarfile,os
ROOT=Path(__file__).resolve().parent.parent
subprocess.run(['cargo','package','--allow-dirty','--locked'],cwd=ROOT,check=True)
archive=ROOT/'target/package/tezbar-markdown-reference-0.2.0.crate'
dest=ROOT/'.build/extracted';dest.mkdir(parents=True,exist_ok=True)
with tarfile.open(archive) as tar:
    for item in tar.getmembers():
        assert not Path(item.name).is_absolute() and '..' not in Path(item.name).parts
    tar.extractall(dest)
package=dest/'tezbar-markdown-reference-0.2.0'
def run(args):subprocess.run(args,cwd=package,check=True)
run(['cargo','test','--locked'])
run(['python3','tools/harness.py'])
run(['cargo','test','--locked','--manifest-path','.build/beta-harness/Cargo.toml'])
run(['pnpm','--dir','tools','install','--frozen-lockfile','--ignore-scripts'])
run(['node','tools/oracle.cjs'])
run(['node','tools/verify.cjs'])
with (package/'.build/views.json').open('w') as output:
    subprocess.run(['cargo','run','--quiet','--locked','--manifest-path','.build/beta-harness/Cargo.toml','--example','views'],cwd=package,stdout=output,check=True)
run(['node','tools/renderer.cjs'])
print('Extracted Cargo package: core, unchanged beta adapter/session, original own-source oracle, full schema/provenance/license hashes, and real beta renderer passed')
