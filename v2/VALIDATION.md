# Validation record

Source: own `6933819bb52944ac5db4c517f932ffa24a95497e`; original Raycast `6734d85992290066fd31a7f5bee3cca3d3545a1a`; beta API `fd300e276356587726566026f072563eb9c34cb1`. Checks ran on 2026-10-24 with Rust/Cargo 1.98.0, Rust 1.85.0, Node 26.7.0 and pnpm 11.23.0. These are local executable checks; neither platform is marked UI-verified.

Passed:

- Current Rust and Rust 1.85: core tests and beta adapter/session integration tests (3 each), formatting checks, strict Clippy with `--all-targets -- -D warnings`. Byte-exact beta source is compiled as a normal test dependency so its unrelated original test modules and OS integration tests are not run.
- Windows GNU `x86_64-pc-windows-gnu`: current Rust `cargo check --all-targets` and strict all-targets Clippy for core and adapter harness. Windows binaries were not executed.
- Executable own TS/TSX/library oracle: all 15 reference topics and 42 basic/additional examples; 232 search cases including every title-word prefix, case, whitespace, Unicode and negative cases; 220 escape-library cases; exact detail Markdown bytes; source `View`/`Back` navigation. The original own modules are transpiled and executed. Only React/Raycast host boundaries are substituted; expected filtering and formatting are not reimplemented in JavaScript.
- Rust lifecycle: each visible action; query restoration on Back; rejection of forged IDs, cross-session IDs, prior-screen IDs, pre-search IDs, discarded staged-search IDs, disposed IDs and relaunched IDs. Heartbeats preserve committed state and IDs. Malformed calls leave actions retryable. Arbitrary event indices, text and URLs never determine displayed content or effects.
- Actual pinned beta `native.rs`: compiled route and annotated session results, native error envelopes, retries after rejection, concurrent search staging, refresh, navigation, disposal and clearing. `common.rs`, `v2_sdk.rs`, `native.rs` and the production adapter compile unchanged. Tauri and routing/installed-command boundaries are in-memory substitutes; desktop effect requests panic in adapter tests. The real SDK's clipboard/open route and negative/transport response preservation are tested with a fake router, without invoking OS effects. Source has no clipboard/browser actions, and no success is claimed for a failed clipboard operation.
- Actual pinned beta renderer: list parser and all row actions, server-filtered results, all 15 Detail/Markdown renders, Back action routing and query restoration, arrow/Enter/Meta+K handler and row-specific action-overlay routing, and actual form-value reconciliation. All renderer/UI/Markdown modules are unchanged; effect calls are forbidden. Examples introduce no external media, script, iframe or generic code-block clipboard UI. These tests exercise handlers and virtual rendering, not physical keyboards or a real desktop window.
- Complete pinned schema validation, negative schema cases, command/platform/credit metadata, 44 pinned fixture hashes, full embedded content parity, exact original logo bytes, full dependency notice hashes, and retained original/own/beta license bytes.
- `cargo package --allow-dirty --locked` and extracted-package core tests, beta harness tests, executable own oracle, full schema/provenance/license validation and actual beta renderer checks. Manifests for nested test crates are shipped as templates because Cargo otherwise excludes nested packages; `tools/harness.py` materializes them inside `.build/`. Relative dependencies resolve entirely within the extracted package.
- Disposable V1 `pnpm test` and `pnpm typecheck`. The tested pnpm dependency resolution is pinned in `tools/v1-validation-pnpm-lock.yaml`. All 20 original tracked files other than the permitted root README link are byte-exact, including root `package-lock.json` (SHA-256 `8dcd0ad592fbda3daef7abcd71f282a1ed579855b400615406bf231af8a0d7ba`).
- Git publication audit: necessary fixture/source/template files are visible to Git despite ignore rules; `.build/`, targets and node_modules are excluded; no staging or commits are needed for validation.

From this `v2/` directory, reproduce the main checks:

```sh
pnpm --dir tools install --frozen-lockfile --ignore-scripts
node tools/oracle.cjs
node tools/verify.cjs
cargo test --locked
cargo +1.85.0 test --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo +1.85.0 clippy --locked --all-targets -- -D warnings
python3 tools/harness.py
cargo test --locked --manifest-path .build/beta-harness/Cargo.toml
cargo +1.85.0 test --locked --manifest-path .build/beta-harness/Cargo.toml
cargo fmt --manifest-path .build/beta-harness/Cargo.toml --check
cargo fmt --manifest-path .build/beta-host/Cargo.toml --check
cargo clippy --locked --manifest-path .build/beta-harness/Cargo.toml --all-targets -- -D warnings
cargo +1.85.0 clippy --locked --manifest-path .build/beta-harness/Cargo.toml --all-targets -- -D warnings
cargo check --locked --all-targets --target x86_64-pc-windows-gnu
cargo clippy --locked --all-targets --target x86_64-pc-windows-gnu -- -D warnings
cargo check --locked --manifest-path .build/beta-harness/Cargo.toml --all-targets --target x86_64-pc-windows-gnu
cargo clippy --locked --manifest-path .build/beta-harness/Cargo.toml --all-targets --target x86_64-pc-windows-gnu -- -D warnings
cargo run --quiet --locked --manifest-path .build/beta-harness/Cargo.toml --example views > .build/views.json
node tools/renderer.cjs
python3 tools/v1.py
python3 tools/package_check.py
```

Runtime data generation is explicit (`node tools/oracle.cjs --generate`), then verified by the non-generating oracle and Rust parity suite. JavaScript tools are validation dependencies only. `.build/` and Cargo targets remain ignored.

The clone transport required recovery after SSH early EOF and failed HTTP/2 and HTTP/1.1 full-pack transfers. A partial metadata clone completed; individual pinned blobs were retrieved before completing checkout. One screenshot was cached from its exact source URL only after its Git blob hash matched. A fresh origin fetch then succeeded before the task branch was created.

XERJ used isolated prefix `ax-0a290da20960`: 7/7 code files indexed, 77 records, exit 3 (`completed-with-junk`). Search found the own command, detail and escape implementation; exhaustive source and publication checks used `rg` and exact Git/blob hashes. The bounded post-change refresh aborted after 0.4 seconds on unsupported frozen dataset/schema evolution for the new Cargo file. Coverage of new V2 files is therefore incomplete; final verification used exhaustive `rg` and byte/hash checks. No global search service was restarted.

Not run: real macOS/Windows launcher UI, physical keyboard behavior, clipboard, browser, window integration, host registry filesystem/install flows, final host application build/registration or deployed verification. Known beta visual/selection/empty-state/shortcut gaps are documented in README. Production integration requires compiled dispatch registration and a rebuild; the JSON manifest alone supplies no execution.
