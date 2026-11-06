#!/usr/bin/env python3
"""Verify and exercise a real extracted package without host installation or OS effects."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import stat
import tempfile
import zipfile

from protocol_check import exercise, malformed_probes


def smoke(path):
    with tempfile.TemporaryDirectory() as temp, zipfile.ZipFile(path) as archive:
        root = Path(temp) / 'package'
        root.mkdir()
        names = archive.namelist()
        assert len(names) <= 4096 and len(names) == len({n.casefold() for n in names}), 'Duplicate/too many ZIP paths'
        assert sum(i.file_size for i in archive.infolist()) <= 512 * 1024 * 1024, 'Expansion limit'
        for item in archive.infolist():
            name = PurePosixPath(item.filename)
            assert not name.is_absolute() and '..' not in name.parts, 'Unsafe ZIP path'
            assert '\\' not in item.filename and ':' not in item.filename, 'Unsafe ZIP path'
            assert not stat.S_ISLNK(item.external_attr >> 16), 'Symlink ZIP entry'
            assert stat.S_ISREG(item.external_attr >> 16), 'Special file ZIP entry'
        manifest = json.loads(archive.read('extension.json'))
        inventory = manifest['files']
        assert set(names) == set(inventory) | {'extension.json'}, 'ZIP/inventory mismatch'
        for name, digest in inventory.items():
            assert hashlib.sha256(archive.read(name)).hexdigest() == digest, 'SHA mismatch: ' + name
        assert manifest['id'] == 'com.tezbar.native.' + manifest['slug']
        assert manifest['schemaVersion'] == 3 and manifest['execution'] == 'process' and manifest['protocolVersion'] == 1
        binary_name = 'bin/tezbar-' + manifest['slug'] + ('.exe' if os.name == 'nt' else '')
        target = 'x86_64-pc-windows-msvc' if os.name == 'nt' else 'aarch64-apple-darwin'
        base = path.parent / (manifest['slug'] + '-' + target)
        artifact = json.loads(Path(str(base) + '.artifact.json').read_text())
        archive_sha = hashlib.sha256(path.read_bytes()).hexdigest()
        assert artifact == {'target': target, 'sha256': archive_sha,
                            'executable': binary_name, 'executableSha256': inventory[binary_name]}
        assert Path(str(path) + '.sha256').read_text().strip() == archive_sha + '  ' + path.name
        assert Path(str(base) + '.executable.sha256').read_text().strip() == inventory[binary_name] + '  ' + binary_name
        assert json.loads(Path(str(base) + '.manifest.json').read_text()) == manifest
        assert archive.getinfo(binary_name).external_attr >> 16 & 0o111, 'Missing executable mode'
        required = ['LICENSE', 'NOTICE', manifest['icon'], 'licenses/dependency-license-index.json',
                    'licenses/OWN-MIT.txt', 'licenses/UPSTREAM-MIT.txt', 'licenses/CC-BY-SA-4.0.txt']
        assert all(name in inventory for name in required)
        notices = json.loads(archive.read('licenses/dependency-license-index.json'))
        assert set(notices['targets']) == {'aarch64-apple-darwin', 'x86_64-pc-windows-msvc'}
        for dependency in notices['dependencies']:
            for name, digest in dependency['documents'].items():
                assert inventory[name] == digest, 'Missing/changed dependency notice: ' + name
        archive.extractall(root)
        binary = root / binary_name
        binary.chmod(0o755)
        # CWD has no original source files, fixtures, SDK checkout, Bun or Node.
        workdir = Path(temp) / 'working'
        workdir.mkdir()
        exercise(binary, manifest, workdir)
        malformed_probes(binary, manifest, workdir)
        print('Extracted ZIP: complete inventory, original assets, both-target notices and process smoke passed')


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('archive', type=Path)
    args = parser.parse_args()
    smoke(args.archive)
