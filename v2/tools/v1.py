"""Test an isolated byte-exact own V1 snapshot with pinned pnpm resolution."""
from pathlib import Path
import shutil,subprocess
ROOT=Path(__file__).resolve().parent.parent
src=ROOT/'fixtures/own';dest=ROOT/'.build/v1';dest.mkdir(parents=True,exist_ok=True)
for item in src.iterdir():
    if item.is_dir():shutil.copytree(item,dest/item.name,dirs_exist_ok=True)
    else:shutil.copy2(item,dest/item.name)
shutil.copy2(ROOT/'tools/v1-validation-pnpm-lock.yaml',dest/'pnpm-lock.yaml')
for command in [['pnpm','install','--frozen-lockfile','--ignore-scripts'],['pnpm','test'],['pnpm','typecheck']]:
    subprocess.run(command,cwd=dest,check=True)
print('Disposable V1 test/typecheck passed; source root and root lockfile untouched')
