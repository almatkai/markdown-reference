use serde_json::{json, Value};
use tezbar_extension_sdk::{BoxFuture, CoreResult, Extension, ExtensionContext};
use tezbar_markdown_reference::{self as engine, Output};

struct MarkdownReference;

// Bound the cloned search draft and its echoed view far below the 16 MiB wire
// limit, including JSON escaping. This is a visible domain failure, not a broken
// transport; the caller can keep its current view, retry, or dispose the session.
const MAX_SEARCH_BYTES: usize = 64 * 1024;

impl Extension for MarkdownReference {
    fn id(&self) -> &'static str {
        engine::ID
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn commands(&self) -> &'static [&'static str] {
        &[engine::COMMAND]
    }

    fn handle<'a>(
        &'a self,
        _ctx: &'a ExtensionContext,
        command: &'a str,
        operation: &'a str,
        payload: &'a Value,
        state: &'a mut Value,
    ) -> BoxFuture<'a, CoreResult> {
        Box::pin(async move {
            // The source has no Host services, preferences, stores or OS effects.
            // SDK owns staging/commit, cancellation, serialization and disposal.
            if operation == "extension:search-text-changed"
                && payload["searchText"]
                    .as_str()
                    .is_some_and(|query| query.len() > MAX_SEARCH_BYTES)
            {
                return Ok(json!({
                    "ok": false,
                    "mode": "no-view",
                    "message": "Search text exceeds the supported 64 KiB UTF-8 limit. Shorten the query and retry."
                }));
            }
            Ok(match engine::handle(command, operation, payload, state)? {
                Output::View { root, actions } => {
                    tezbar_extension_sdk::view(engine::TITLE, root, actions)
                }
                Output::Unchanged => tezbar_extension_sdk::unchanged(),
                Output::Disposed(active) => json!(active),
            })
        })
    }
}

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("extension runtime");
    let result = runtime.block_on(tezbar_extension_sdk::serve(MarkdownReference));
    runtime.shutdown_timeout(std::time::Duration::from_millis(500));
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
