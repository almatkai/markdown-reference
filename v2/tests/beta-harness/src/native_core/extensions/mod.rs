#[rustfmt::skip]
#[path = "../../../../../fixtures/beta/src-tauri/src/native_core/extensions/v2_sdk.rs"]
pub mod v2_sdk;
#[path = "../../../../../adapter/v2_markdown_reference.rs"]
pub mod v2_markdown_reference;
#[rustfmt::skip]
#[path = "../../../../../fixtures/beta/src-tauri/src/native_core/extensions/native.rs"]
mod native;
use crate::native_core::common::{CoreContext, HandlerResult};
use serde_json::Value;
use std::sync::{Arc, Mutex};
#[derive(Clone, Default)]
pub struct Sessions(native::NativeSessions);
impl Sessions {
    pub async fn handle(&self, ctx: &CoreContext, channel: &str, args: &[Value]) -> HandlerResult {
        self.0
            .handle(ctx, &Arc::new(Mutex::new(())), channel, args)
            .await
    }
    pub async fn clear(&self) {
        self.0.clear().await;
    }
}
// The only routing seams: this command's compiled dispatch and in-memory
// installed-command/preferences lookup. Filesystem discovery is outside this
// harness; native.rs itself (locking/staging/annotation/cancellation) is exact.
mod v2_runtime {
    use super::*;
    use crate::native_core::common::CoreResult;
    pub(super) fn slug(id: &str) -> Option<&str> {
        let slug = id
            .strip_prefix("com.tezbar.native.")
            .or_else(|| id.strip_prefix("raycast."))
            .unwrap_or(id);
        (slug == "markdown-reference").then_some(slug)
    }
    pub(super) fn supports(id: &str, command: &str) -> bool {
        slug(id).is_some() && super::v2_markdown_reference::commands().contains(&command)
    }
    pub(super) async fn handle(
        ctx: &CoreContext,
        slug: &str,
        command: &str,
        op: &str,
        payload: &Value,
        state: &mut Value,
    ) -> CoreResult {
        if slug != "markdown-reference" {
            return Err("Unknown compiled route".into());
        }
        super::v2_markdown_reference::handle(ctx, command, op, payload, state).await
    }
}
mod registry {
    use super::*;
    use crate::native_core::common::CoreResult;
    pub(super) struct Installed;
    pub(super) fn installed_command(
        _ctx: &CoreContext,
        id: &str,
        command: &str,
    ) -> Option<Installed> {
        super::v2_runtime::supports(id, command).then_some(Installed)
    }
    pub(super) fn preferences(_installed: &Installed, _command: Option<&str>) -> CoreResult {
        Ok(serde_json::json!({}))
    }
}
