# Native package validation contract

The production entry point is `native-src/main.rs`. It depends on the complete
owned `v2/src/lib.rs` and its embedded `v2/data/reference.json`, plus SDK revision
`aaec571673ab64a85a136cf6bd054ef660c4d93c`. Root `Cargo.lock` is authoritative for
the process executable. The separate preserved v2 lockfile and beta fixtures are
historical validation inputs, not runtime provisioning. Root Raycast sources,
command names, preferences (none), icons and licenses are preserved.

`cargo +1.85.0 test --locked -j2` runs the original three engine parity tests
directly from `v2/tests/parity.rs`, including byte comparisons with the pinned
TypeScript oracle. No frozen host adapter is involved in production.

`scripts/smoke-package.py` checks every archive member against its exact SHA256
inventory, validates the package layout, executable permissions, both targets'
dependency notices and preserved attribution. It extracts to a temporary path,
then runs the executable from an unrelated directory with a minimal environment
and no JS toolchain on PATH. `scripts/protocol_check.py` exercises the same runner
directly if a ZIP is not yet available. Test-only oracle data is read by Python
from this repository; the extracted executable needs no source tree or data files.

The recording transport grants no host capabilities and rejects/records every
reverse request. This source has no Host services, effects, forms or persisted
data, so no clipboard/browser effects, acknowledgements, draft-field callbacks or
migration imports are fabricated. Tests cover protocol info, hello, initialize,
actual native `session.open`/`session.call` UI payload shapes, all 15 topics and
oracle searches, byte-exact multiline Markdown, View/Back draft restoration,
row `actionIds`, `backActionId`, protocol errors and retry, stale/forged/cross-session
actions, cancelled/superseded search, refresh/load-more, 128 live sessions plus
limit recovery, disposal and shutdown with stdin still open. Separate child
processes test malformed JSON, unknown envelope type, over-16-MiB frames and EOF.

Search text is bounded to 64 KiB UTF-8. Even worst-case JSON escaping stays far
below the 16 MiB frame budget. An oversized query returns protocol success with
`{ok:false,mode:"no-view",message:...}` and leaves current state/action bindings
intact; retry and explicit disposal are tested. Forged actions and malformed calls
remain protocol errors. The original dataset has no fallible host effects or
expected domain failures otherwise. Valid actions always return a full `view`;
only refresh and load-more return `unchanged`, which the accepted host handles.

Host references were inspected read-only at accepted commit
`d77f03b12a93d72cd2a56a73faa1a04fae94a2cd`: `native.rs` stages protocol calls and
annotates views; `ExtensionRuntimeView.tsx` consumes `ok`, `mode`, `message`, root
and actions; `raycast-api/index.tsx` routes Detail Back using `backActionId`;
`list-runtime.tsx` uses server search and each row's `actionIds`. This package
requires no host features added after that baseline. In particular it has no
secondary form-submit menu or ordinary form callbacks, and does not rely on
host changes currently under review. Wire tests mirror the caller's payloads;
they do not execute the real desktop renderer.
The accepted renderer resets List selection after returning from Detail and uses
the generic “No matching applications” label for nonempty searches with no results.
Its Meta+K action-menu shortcut and platform rendering are host presentation
limitations; this package does not silently assume a pending host fix.

Packaging verifies the published SDK pin, target executable format/architecture,
identity/version/protocol/commands, and exact upstream notices for the locked
normal/build dependency closure of macOS arm64 and Windows x64. It vendors all
license alternatives, copyright and notice documents, including nested documents
and UNLICENSE, byte for byte. `scripts/vendor-licenses.py --check` fails if notices
or the dependency index are missing/stale. `resources/assets/` retains both
original icons without recompression. There is no new workflow or CI build.

Remaining acceptance gates are independent code review, centrally coordinated
Windows Rust 1.85 build and extracted-binary checks, trusted artifact pins, and
real host installation/UI/keyboard QA on both platforms. These tests never launch
the user's desktop, browser or clipboard. A successful headless smoke is not
desktop QA. Source manifest verification therefore remains `not-run` with durable
scope notes; dated local evidence belongs in the implementation handoff JSON.
