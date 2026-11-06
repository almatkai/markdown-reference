# Markdown Reference

Search a compact Markdown syntax reference from Tezbar.

## Installable native Rust version

The root `extension.json` describes the standalone Rust process package
`com.tezbar.native.markdown-reference`, replacing `raycast.markdown-reference`.
Its command remains **Search Markdown Syntax** (`markdown-reference`) for macOS
arm64 and Windows x64. The original Raycast TypeScript implementation, manifest,
icons and licenses remain in `src/`, `package.json`, `assets/` and `LICENSE`.

The native executable reuses the complete [v2 engine](v2/src/lib.rs): all 15 topics,
source order, case-insensitive prefix matching on the first two title words,
exact escaped Markdown examples and additional examples, and View/Back navigation
that restores the search draft. HTML fields stay data and are not rendered.
There are no source preferences, forms, stored data or clipboard/browser actions;
no migration paths or host capability grants are needed. The SDK owns JSONL
lifecycle, transactional session state, cancellation and bounded resources.
There is no app/Tauri dependency, runtime fixture materialization, JS subprocess,
or network dependency. Valid actions return full views, and only refresh/load-more
return unchanged. Invalid or stale actions fail without consuming current actions.

Build with Rust 1.85 or newer, using the locked SDK git revision:

```sh
export CARGO_BUILD_JOBS=2
export CARGO_TARGET_DIR="$PWD/target"
cargo +1.85.0 test --locked -j2
cargo +1.85.0 build --release --locked -j2 --target aarch64-apple-darwin
python3 scripts/vendor-licenses.py --check
python3 scripts/package.py --target aarch64-apple-darwin \
  --binary target/aarch64-apple-darwin/release/tezbar-markdown-reference
python3 scripts/smoke-package.py dist/markdown-reference-aarch64-apple-darwin.zip
```

On Windows use `x86_64-pc-windows-msvc` and the `.exe` binary, then run the same
package/smoke scripts on Windows. Packaging checks the immutable SDK pin, both
targets' dependency notices, binary identity and command coverage, then writes a
root-layout ZIP containing `extension.json`, `bin/tezbar-markdown-reference[.exe]`,
`resources/`, `licenses/`, `LICENSE` and `NOTICE`. The generated manifest inventories
every file except itself. ZIP and executable SHA256 sidecars, artifact metadata
and the resolved manifest are written beside the ZIP. The source manifest's empty
`files` and `artifacts` are filled during packaging/release; source metadata alone
is not an installable release. See [native validation contract](NATIVE_VALIDATION.md).

A trusted release manifest must pin the real target ZIP URL, archive SHA256,
executable path and executable SHA256. An authorized publisher/host catalog owner
must supply those pins after independent review and both platform checks. Local
import uses a schema-3/process-1 capable Tezbar host:

```sh
tezbar --extensions-v3-import /absolute/trusted-release-manifest.json /absolute/package.zip
```

Close and reopen Tezbar after import. This package uses the accepted host contract
at `d77f03b12a93d72cd2a56a73faa1a04fae94a2cd`; it requires no new host changes.
Real desktop install, rendering and keyboard QA remain separate acceptance gates
on each platform. Headless protocol tests do not establish desktop acceptance.
List selection after Back and Markdown appearance follow the host renderer.
The accepted renderer uses the generic “No matching applications” label for a
nonempty query with no results, and its action-menu shortcut is Meta+K; native
platform keyboard behavior still needs desktop QA.
Search queries are bounded to 64 KiB of UTF-8; longer queries return a visible
failure without replacing the current view or invalidating its actions.

[Native V2 Rust implementation and integration](v2/README.md)
