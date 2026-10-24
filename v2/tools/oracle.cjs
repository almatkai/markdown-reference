// Execute the pinned OWN source and library; only Raycast/React host boundaries
// are replaced. Expected content, filtering and Markdown are never rewritten.
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const assert = require('node:assert/strict');
const ts = require('typescript');
const root = path.resolve(__dirname, '..');
const cache = new Map();
const hooks = [];
let hook = 0;
const stack = [];
const node = (type, props) => ({ type, props });
const React = { useState(initial) { const i = hook++; if (!(i in hooks)) hooks[i] = initial; return [hooks[i], value => hooks[i] = value]; } };
const raycast = {
  List: Object.assign('List', {}),
  Detail: 'Detail',
  ActionPanel: Object.assign(function ActionPanel() {}, { Item: 'ActionPanel.Item' }),
  useNavigation: () => ({ push: view => stack.push(view), pop: () => stack.pop() }),
};
// String primitives cannot carry Item on a string value.
raycast.List = Object.assign(function List() {}, { Item: 'List.Item' });
function load(filename) {
  if (cache.has(filename)) return cache.get(filename).exports;
  const module = { exports: {} }; cache.set(filename, module);
  const source = fs.readFileSync(filename, 'utf8');
  const output = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022, jsx: ts.JsxEmit.ReactJSX } }).outputText;
  function req(name) {
    if (name === '@raycast/api') return raycast;
    if (name === 'react') return React;
    if (name === 'react/jsx-runtime') return { jsx: node, jsxs: node };
    if (name.startsWith('.')) {
      const base = path.resolve(path.dirname(filename), name);
      return load(['.ts', '.tsx'].map(ext => base + ext).find(p => fs.existsSync(p)));
    }
    throw Error('Unexpected own-source dependency: ' + name);
  }
  vm.runInThisContext('(function(require,module,exports){' + output + '\n})', { filename })(req, module, module.exports);
  return module.exports;
}
const src = path.join(root, 'fixtures/own/src');
const references = load(path.join(src, 'markdownReference.ts')).default;
const Component = load(path.join(src, 'markdown-reference.tsx')).default;
const escape = load(path.join(src, 'markdown-escape.ts')).default;
function render() { hook = 0; return Component(); }
function rows(tree) { const children = tree.props.children; return Array.isArray(children) ? children : []; }
let list = render();
assert.equal(rows(list).length, 15);
const queries = new Set(['', ' ', '  ', 'li', 'LI', 'list', 'Lists', 'LISTS', 'b', 'B', 'rules', 'Rules', 'ith', 'list ', 'ordered lists', 'horiz', 's', 'Σ', 'İ', 'K', '😀', '日本', 'a\0b', 'blockquote\n']);
for (const ref of references) {
  for (const word of ref.name.split(' ')) {
    for (let i = 0; i <= word.length; i++) {
      queries.add(word.slice(0, i)); queries.add(word.slice(0, i).toUpperCase());
    }
  }
}
const searches = [];
for (const query of queries) {
  render().props.onSearchTextChange(query);
  list = render();
  searches.push({ query, names: rows(list).map(row => row.props.title) });
  for (const row of rows(list)) {
    const before = hooks[0];
    row.props.actions.props.children.props.onAction();
    const pushed = stack.at(-1);
    const detail = pushed.type(pushed.props);
    assert.equal(detail.type, 'Detail');
    assert.equal(detail.props.actions.props.children.props.title, 'Back');
    detail.props.actions.props.children.props.onAction();
    assert.equal(stack.length, 0);
    assert.equal(hooks[0], before); // Original Back preserves filtered list/draft.
  }
}
render().props.onSearchTextChange('');
const markdowns = rows(render()).map(row => {
  row.props.actions.props.children.props.onAction();
  const pushed = stack.at(-1); const detail = pushed.type(pushed.props);
  detail.props.actions.props.children.props.onAction();
  return detail.props.markdown;
});
const escapeCases = [];
for (const text of ['*#+/()[]<>_=-`\\ &amp; 😀\r\n', '', ...references.flatMap(ref => [...ref.examples, ...ref.additional_examples].map(ex => ex.markdown))]) {
  for (const skips of [[], ['parentheses'], ['plus sign', 'number signs'], ['angle brackets', 'asterisks'], ['hyphen', 'slashes']]) {
    escapeCases.push({ text, skips, expected: escape(text, skips) });
  }
}
const serial = value => JSON.parse(JSON.stringify(value));
const data = JSON.stringify(serial(references), null, 2) + '\n';
const oracle = JSON.stringify(serial({ source: '6933819bb52944ac5db4c517f932ffa24a95497e', references, searches, markdowns, escapeCases }), null, 2) + '\n';
if (process.argv.includes('--generate')) {
  fs.writeFileSync(path.join(root, 'data/reference.json'), data);
  fs.writeFileSync(path.join(root, 'tests/oracle.json'), oracle);
} else {
  assert.equal(fs.readFileSync(path.join(root, 'data/reference.json'), 'utf8'), data);
  assert.equal(fs.readFileSync(path.join(root, 'tests/oracle.json'), 'utf8'), oracle);
}
console.log(`OWN oracle: ${references.length} topics, ${searches.length} searches, ${escapeCases.length} escape cases; all source View/Back paths executed`);
