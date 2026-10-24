# Compiled integration with the pinned beta

Use `fd300e276356587726566026f072563eb9c34cb1` as the API reference. Integration changes belong in the host application. The standalone library and adapter are already compiled together by the supplied harness; no host application files are changed by this package.

1. Vendor this `v2/` directory at `src-tauri/native-extensions/markdown-reference/`. Retain the embedded data, provenance and notices. Add this host Cargo dependency:

   ```toml
   tezbar-markdown-reference = { path = "native-extensions/markdown-reference" }
   ```

2. Register the unchanged adapter in `src-tauri/src/native_core/extensions/mod.rs`:

   ```rust
   #[path = "../../../native-extensions/markdown-reference/adapter/v2_markdown_reference.rs"]
   mod v2_markdown_reference;
   ```

3. Extend the existing compiled router in `src-tauri/src/native_core/extensions/v2_runtime.rs`. Add `"markdown-reference"` to the accepted slug match; add the following arm to `commands`:

   ```rust
   "markdown-reference" => super::v2_markdown_reference::commands(),
   ```

   Add this arm to the existing async `handle` match:

   ```rust
   "markdown-reference" => {
       super::v2_markdown_reference::handle(ctx, command, operation, payload, state).await
   }
   ```

   The existing slug normalizer then handles the canonical ID, legacy replacement ID, and slug. Keep `supports` dependent on compiled commands. Use the existing `native.rs` session lifecycle, payload preferences, staged searches, cancellation and disposal.

4. Append the complete object from `extension-v2.json` to the host `v2_catalog.json` array. Do not add a root author field, change verification to `pass`, or copy legacy profile data. The existing native installer writes the managed manifest/marker and installed database entry after compiled dispatch is available. Keep the legacy package for fallback.

5. Copy `assets/markdown-reference.png` unchanged to **`src/renderer/public/extensions/icons/markdown-reference.png`**. The public URL is `/extensions/icons/markdown-reference.png`; root `public/` is not this host's renderer asset directory. Verify its SHA-256 after copying.

6. Check the integrated host, rebuild the Rust application and renderer, and run `node scripts/extensions-v2-list.mjs` using the rebuilt native CLI. It must report the command from compiled dispatch. Install using the existing extension UI/CLI flow, launch the canonical command, and perform real macOS and Windows UI/keyboard QA. Update each platform's verification only with an actual QA artifact.

The adapter's `handle` signature matches beta `CoreContext`, `CoreResult`, state and payload types. The harness compiles the production adapter unchanged against byte-exact beta `common.rs`, `v2_sdk.rs` and `native.rs`. Only Tauri and routing boundaries (one compiled command, in-memory installed-command/preferences lookup and effect router) are substituted. It does not test host filesystem discovery, installation, packaging of the final application, or real OS effects.
