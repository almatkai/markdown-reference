#!/usr/bin/env python3
"""Exercise the real process with a recording transport that denies every host RPC."""
import argparse
import json
import os
from pathlib import Path
import queue
import subprocess
import tempfile
import threading
import uuid

ROOT = Path(__file__).resolve().parents[1]
MAX_FRAME = 16 * 1024 * 1024
COMMAND = 'markdown-reference'


class RecordingHost:
    def __init__(self, binary, manifest, root):
        self.manifest = manifest
        self.root = Path(root)
        self.package_dir = binary.resolve().parent.parent
        self.calls = []
        self.inbox = queue.Queue()
        self.pending = {}
        # No Node/Bun, credentials, user profile or GUI service addresses.
        env = {key: value for key, value in os.environ.items()
               if key in ('SystemRoot', 'WINDIR', 'TEMP', 'TMP')}
        env['PATH'] = (os.path.join(env.get('SystemRoot', 'C:/Windows'), 'System32')
                       if os.name == 'nt' else '/usr/bin:/bin')
        self.process = subprocess.Popen([str(binary.resolve())], cwd=root, env=env,
                                        stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                        stderr=subprocess.PIPE)
        self.reader = threading.Thread(target=self.read, daemon=True)
        self.reader.start()
        hello = self.receive()
        assert hello['type'] == 'hello', hello
        assert hello['id'] == manifest['id'] and hello['version'] == manifest['version'], hello
        assert hello['protocol'] == 1 and hello['sdkVersion'] == '0.1.1', hello

    def read(self):
        try:
            while True:
                line = self.process.stdout.readline(MAX_FRAME + 2)
                if not line:
                    self.inbox.put(EOFError('Unexpected protocol EOF'))
                    return
                assert line.endswith(b'\n') and len(line) <= MAX_FRAME + 1, 'Invalid frame budget'
                self.inbox.put(json.loads(line))
        except Exception as error:
            self.inbox.put(error)

    def receive(self):
        frame = self.inbox.get(timeout=15)
        if isinstance(frame, Exception):
            raise frame
        if frame.get('type') == 'request':
            self.calls.append(frame)
            self.write({'type': 'response', 'id': frame['id'],
                        'error': {'code': 'denied', 'message': 'No granted host services'}})
            raise AssertionError('Unexpected host effect: ' + str(frame))
        return frame

    def write(self, frame):
        encoded = json.dumps(frame, ensure_ascii=False, separators=(',', ':')).encode()
        assert len(encoded) <= MAX_FRAME
        self.process.stdin.write(encoded + b'\n')
        self.process.stdin.flush()

    def send(self, method, params):
        ident = 'h:' + str(uuid.uuid4())
        self.write({'type': 'request', 'id': ident, 'method': method, 'params': params})
        return ident

    def response(self, ident):
        while ident not in self.pending:
            frame = self.receive()
            assert frame['type'] == 'response', frame
            assert frame['id'] not in self.pending, 'Duplicate response'
            self.pending[frame['id']] = frame
        return self.pending.pop(ident)

    def request(self, method, params, error=None):
        frame = self.response(self.send(method, params))
        if error is not None:
            assert 'error' in frame and error.lower() in frame['error']['message'].lower(), frame
            return frame['error']
        assert 'error' not in frame and 'result' in frame, frame
        return frame['result']

    def initialize(self):
        result = self.request('initialize', {
            'protocol': 1, 'extensionId': self.manifest['id'],
            'platform': 'Windows' if os.name == 'nt' else 'macOS',
            'packageDir': str(self.package_dir), 'dataDir': str(self.root / 'data'),
            'tempDir': str(self.root / 'temp'), 'migrationPaths': {},
            'grantedCapabilities': [], 'reason': 'command',
        })
        assert result['ready'] is True, result

    def open(self, session, error=None, **extra):
        return self.request('session.open', {
            'sessionId': session, 'commandName': COMMAND, 'preferences': {},
            'argumentValues': {}, 'searchText': None, 'launchContext': None, **extra,
        }, error)

    def call(self, session, operation, payload=None, error=None):
        # Payloads match the native caller, which passes the UI body unchanged.
        return self.request('session.call', {
            'sessionId': session, 'operation': 'extension:' + operation,
            'payload': {'sessionId': session, **(payload or {})},
        }, error)

    def dispose(self, session):
        assert self.request('session.dispose', {'sessionId': session}) is True

    def finish(self):
        self.request('shutdown', {})
        assert self.process.wait(timeout=5) == 0
        assert not self.calls, self.calls

    def close(self):
        if self.process.poll() is None:
            self.process.kill()
            self.process.wait(timeout=5)
        self.reader.join(timeout=5)
        stderr = self.process.stderr.read().decode(errors='replace')
        self.process.stdin.close()
        self.process.stdout.close()
        self.process.stderr.close()
        return stderr


def list_view(result, names, query=''):
    assert result['ok'] is True and result['mode'] == 'view', result
    assert result['title'] == 'Search Markdown Syntax'
    root = result['root']
    assert root['type'] == 'List' and root['props']['__hasServerSearch'] is True
    assert root['props']['searchText'] == query
    assert [row['props']['title'] for row in root['children']] == names
    actions = {a['id']: a for a in result['actions']}
    assert len(actions) == len(names)
    for row in root['children']:
        ids = row['props']['actionIds']
        assert len(ids) == 1 and actions[ids[0]]['title'] == 'View'
        assert actions[ids[0]]['kind'] == 'push'


def exercise(binary, manifest, workdir):
    oracle = json.loads((ROOT / 'v2/tests/oracle.json').read_text(encoding="utf-8"))
    names = [r['name'] for r in oracle['references']]
    info = json.loads(subprocess.check_output([str(binary.resolve()), '--protocol-info'], timeout=10))
    assert info == {'id': manifest['id'], 'version': manifest['version'], 'protocol': 1,
                    'sdkVersion': '0.1.1', 'commands': [COMMAND]}, info
    assert json.loads(subprocess.check_output([str(binary.resolve()), '--self-test'], timeout=10)) == info
    assert manifest['capabilities'] == [] and manifest['preferences'] == []
    host = RecordingHost(binary, manifest, workdir)
    try:
        host.open('preinitialize', error='Not initialized')
        host.initialize()
        host.request('initialize', {}, error='Already initialized')
        host.open('unknown', commandName='forged', error='Unknown command')
        current = host.open('main')
        list_view(current, names)
        host.open('main', error='Session exists')
        for malformed in [{}, {'actionId': None}, {'actionId': 1}, {'actionId': '__nav_pop__'},
                          {'actionId': 'copy', 'url': 'file:///forged'}, {'actionId': 'forged'}]:
            host.call('main', 'invoke-action', malformed, error='action')
        for index, name in enumerate(names):
            view_id = current['root']['children'][index]['props']['actionIds'][0]
            detail = host.call('main', 'invoke-action', {
                'actionId': view_id,
                'formValues': {'index': 999, 'text': 'forged', 'url': 'https://attacker.invalid'},
            })
            assert detail['ok'] is True and detail['mode'] == 'view'
            assert detail['root']['type'] == 'Detail'
            markdown = detail['root']['props']['markdown']
            assert markdown == oracle['markdowns'][index] and '\n' in markdown, name
            back_id = detail['root']['props']['backActionId']
            assert detail['actions'] == [{'id': back_id, 'title': 'Back', 'kind': 'pop'}]
            host.call('main', 'invoke-action', {'actionId': view_id}, error='stale')
            host.call('main', 'search-text-changed', {'searchText': 'overwrite'}, error='only available')
            for op in ('refresh-session', 'load-more'):
                assert host.call('main', op) == {'ok': True, 'mode': 'unchanged'}
            current = host.call('main', 'invoke-action', {'actionId': back_id})
            list_view(current, names)
            host.call('main', 'invoke-action', {'actionId': back_id}, error='stale')
        # All pinned source queries including whitespace, Unicode, no results and case.
        for case in oracle['searches']:
            previous_ids = [a['id'] for a in current['actions']]
            current = host.call('main', 'search-text-changed', {'searchText': case['query']})
            list_view(current, case['names'], case['query'])
            if previous_ids:
                host.call('main', 'invoke-action', {'actionId': previous_ids[0]}, error='stale')
            if case['names']:
                detail = host.call('main', 'invoke-action', {'actionId': current['actions'][0]['id']})
                current = host.call('main', 'invoke-action', {'actionId': detail['root']['props']['backActionId']})
                list_view(current, case['names'], case['query'])
        # Protocol failures preserve the current valid bindings, then same-session retry.
        current = host.call('main', 'search-text-changed', {'searchText': 'LI'})
        action = current['actions'][0]['id']
        limit_error = host.call('main', 'search-text-changed', {'searchText': 'x' * (64 * 1024 + 1)})
        assert limit_error['ok'] is False and limit_error['mode'] == 'no-view'
        assert '64 KiB' in limit_error['message']
        host.call('main', 'search-text-changed', {'searchText': 17}, error='Missing searchText')
        host.call('main', 'unsupported', error='Unsupported session operation')
        host.request('session.call', {'sessionId': 'main', 'operation': 'extension:invoke-action',
                                     'payload': []}, error='payload must be object')
        other = host.open('other')
        host.call('other', 'invoke-action', {'actionId': action}, error='stale')
        host.call('main', 'invoke-action', {'actionId': other['actions'][0]['id']}, error='stale')
        host.dispose('other')
        detail = host.call('main', 'invoke-action', {'actionId': action})
        current = host.call('main', 'invoke-action', {'actionId': detail['root']['props']['backActionId']})
        list_view(current, [r['name'] for r in oracle['references']
                           if any(w.startswith('li') for w in r['name'].lower().split(' ')[:2])], 'LI')
        # Accepted maximum plus worst-case JSON escaping stays well below one frame.
        maximum = '\x00' * (64 * 1024)
        bounded = host.call('main', 'search-text-changed', {'searchText': maximum})
        list_view(bounded, [], maximum)
        assert len(json.dumps(bounded).encode()) < MAX_FRAME // 8
        host.open('limit-error')
        limit_error = host.call('limit-error', 'search-text-changed', {'searchText': 'é' * (32 * 1024 + 1)})
        assert limit_error['ok'] is False and limit_error['mode'] == 'no-view'
        host.dispose('limit-error')  # Actual caller cleanup must work after a domain failure.
        # A reader-ordered replacement wins over any completed/cancelled earlier search.
        requests = [host.send('session.call', {
            'sessionId': 'main', 'operation': 'extension:search-text-changed',
            'payload': {'sessionId': 'main', 'searchText': 'bold' if i % 2 else 'heading'},
        }) for i in range(40)]
        host.write({'type': 'cancel', 'requestId': requests[-1], 'reason': 'new search'})
        latest = host.send('session.call', {'sessionId': 'main',
            'operation': 'extension:search-text-changed',
            'payload': {'sessionId': 'main', 'searchText': 'LI'}})
        superseded_actions = []
        for request in requests:
            frame = host.response(request)
            if 'result' in frame:
                superseded_actions.extend(frame['result']['actions'])
            else:
                assert any(s in frame['error']['message'].lower()
                           for s in ('superseded', 'cancelled')), frame
        current = host.response(latest)['result']
        assert current['root']['props']['searchText'] == 'LI'
        for action in superseded_actions:
            host.call('main', 'invoke-action', {'actionId': action['id']}, error='stale')
        detail = host.call('main', 'invoke-action', {'actionId': current['actions'][0]['id']})
        assert detail['mode'] == 'view'
        host.dispose('main')
        host.call('main', 'invoke-action', {'actionId': detail['actions'][0]['id']}, error='Unknown session')
        host.open('fresh')
        host.call('fresh', 'invoke-action', {'actionId': detail['actions'][0]['id']}, error='stale')
        host.dispose('fresh')
        # SDK limit is real; disposing always releases a slot for the next session.
        for i in range(128):
            host.open('limit-' + str(i))
        host.open('overflow', error='session limit')
        host.dispose('limit-0')
        host.open('replacement')
        host.dispose('replacement')
        for i in range(1, 128):
            host.dispose('limit-' + str(i))
        host.finish()
        assert not (Path(workdir) / 'data').exists(), 'Stateless command wrote a store'
    finally:
        stderr = host.close()
        assert not stderr, stderr
    print('Protocol: handshake; all 15 topics/oracle queries; action validation; multiline detail; '
          'View/Back; retry; cross-session isolation; cancel/latest search; frame/search budgets; 128-session limit; disposal; '
          'shutdown; zero host RPCs passed')


def malformed_probes(binary, manifest, workdir):
    # Separate processes: these transport failures must never be mistaken for domain outcomes.
    for raw, expected in ((b'{bad json}\n', 'json'),
                          (b'{"type":"invented"}\n', 'Unknown frame type'),
                          (b'x' * (MAX_FRAME + 1) + b'\n', '16 MiB')):
        host = RecordingHost(binary, manifest, workdir)
        try:
            try:
                host.process.stdin.write(raw)
                host.process.stdin.flush()
            except BrokenPipeError:
                pass
            assert host.process.wait(timeout=10) != 0, 'Malformed protocol was accepted'
            stderr = host.process.stderr.read().decode(errors='replace')
            assert expected.lower() in stderr.lower(), stderr
        finally:
            host.close()
    # EOF must clean up live sessions and exit without requiring shutdown.
    host = RecordingHost(binary, manifest, workdir)
    try:
        host.initialize()
        host.open('eof')
        host.process.stdin.close()
        assert host.process.wait(timeout=5) == 0
    finally:
        assert not host.close()
    print('Transport: malformed JSON/type, >16 MiB frame rejection and EOF cleanup passed')


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('binary', type=Path)
    args = parser.parse_args()
    manifest = json.loads((ROOT / 'extension.json').read_text(encoding="utf-8"))
    with tempfile.TemporaryDirectory() as temp:
        exercise(args.binary, manifest, temp)
        malformed_probes(args.binary, manifest, temp)
