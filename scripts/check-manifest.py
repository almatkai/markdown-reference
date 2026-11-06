#!/usr/bin/env python3
"""Validate source and generated manifests against an explicitly supplied host schema."""
import argparse
import hashlib
import json
from pathlib import Path
import jsonschema

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('--schema', required=True, type=Path)
parser.add_argument('manifests', nargs='*', type=Path, default=[ROOT / 'extension.json'])
args = parser.parse_args()
schema = json.loads(args.schema.read_text())
jsonschema.Draft202012Validator.check_schema(schema)
source = json.loads((ROOT / 'package.json').read_text())
for path in args.manifests:
    manifest = json.loads(path.read_text())
    validator = jsonschema.Draft202012Validator(schema, format_checker=jsonschema.FormatChecker())
    errors = list(validator.iter_errors(manifest))
    if errors:
        raise SystemExit('; '.join(error.message for error in errors))
    assert manifest['replaces'] == ['raycast.' + source['name']]
    assert manifest['id'] == 'com.tezbar.native.' + source['name']
    assert manifest['preferences'] == source.get('preferences', [])
    assert manifest['platforms'] == source['platforms']
    assert [c['name'] for c in manifest['commands']] == [c['name'] for c in source['commands']]
    for command, original in zip(manifest['commands'], source['commands']):
        for key, value in original.items():
            assert command[key] == value, (key, value)
        assert command['runtime'] == 'rust' and command['refreshMs'] is None
    assert manifest['capabilities'] == []
    print('Schema 3 and complete source command/preference metadata match:', path.name)
for name in ('logo.png', 'list-icon.png'):
    assert (ROOT / 'assets' / name).read_bytes() == (ROOT / 'resources/assets' / name).read_bytes()
    print('Original icon preserved:', name, hashlib.sha256((ROOT / 'assets' / name).read_bytes()).hexdigest())
