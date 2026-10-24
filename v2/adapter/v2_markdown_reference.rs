//! Compiled beta adapter. Vendor this file unchanged beside the host v2_sdk.
use crate::native_core::common::{CoreContext, CoreResult};
use serde_json::{json, Value};
use tezbar_markdown_reference::{self as core, Output};

pub fn commands() -> &'static [&'static str] {
    &[core::COMMAND]
}

pub async fn handle(
    _ctx: &CoreContext,
    command: &str,
    operation: &str,
    payload: &Value,
    state: &mut Value,
) -> CoreResult {
    Ok(match core::handle(command, operation, payload, state)? {
        Output::View { root, actions } => super::v2_sdk::view(core::TITLE, root, actions),
        Output::Unchanged => super::v2_sdk::unchanged(),
        Output::Disposed(active) => json!(active),
    })
}
