"""Materialize test crate templates locally (Cargo omits nested packages)."""
from pathlib import Path
import shutil
ROOT = Path(__file__).resolve().parent.parent
for name in ['beta-host', 'beta-harness', 'tauri-stub']:
    source = ROOT / 'tests' / name
    dest = ROOT / '.build' / name
    dest.mkdir(parents=True, exist_ok=True)
    for item in source.iterdir():
        if item.name == 'target':
            continue
        target = dest / item.name.replace('Cargo.template.toml', 'Cargo.toml').replace('Cargo.template.lock', 'Cargo.lock')
        if item.is_dir():
            shutil.copytree(item, target, dirs_exist_ok=True)
        else:
            shutil.copy2(item, target)
print('.build/beta-harness/Cargo.toml')
