use markdown_reference_beta_harness::{
    native_core::{
        common::{CoreContext, CorePaths},
        extensions::{v2_markdown_reference as adapter, v2_sdk},
    },
    Sessions,
};
use serde_json::{json, Value};
use std::sync::Arc;
fn context() -> CoreContext {
    let p = std::path::PathBuf::from("/forbidden-test-profile");
    let ctx = CoreContext::new(
        CorePaths {
            data_dir: p.clone(),
            config_dir: p.clone(),
            resource_dir: p.clone(),
            temp_dir: p.clone(),
            home_dir: p,
        },
        None,
    );
    ctx.set_router(Arc::new(|channel, _| {
        Box::pin(async move { panic!("Unexpected OS request: {channel}") })
    }))
    .unwrap();
    ctx
}
async fn ipc(s: &Sessions, c: &CoreContext, ch: &str, body: Value) -> Value {
    s.handle(c, ch, &[body]).await.unwrap().unwrap()
}
fn launch_body() -> Value {
    json!({"extensionId":"com.tezbar.native.markdown-reference","commandName":"markdown-reference"})
}
#[tokio::test]
async fn unchanged_adapter_compiles_with_exact_sdk_and_refresh_never_changes_capabilities() {
    let ctx = context();
    let mut s = json!({});
    let v = adapter::handle(
        &ctx,
        "markdown-reference",
        "extension:run-command",
        &json!({}),
        &mut s,
    )
    .await
    .unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["actions"].as_array().unwrap().len(), 15);
    let committed = s.clone();
    let mut cloned = s.clone();
    assert_eq!(
        adapter::handle(
            &ctx,
            "markdown-reference",
            "extension:refresh-session",
            &json!({}),
            &mut cloned
        )
        .await
        .unwrap(),
        v2_sdk::unchanged()
    );
    assert_eq!(committed, cloned);
    let staged = adapter::handle(
        &ctx,
        "markdown-reference",
        "extension:search-text-changed",
        &json!({"searchText":"Bold"}),
        &mut cloned,
    )
    .await
    .unwrap();
    let id = &staged["actions"][0]["id"];
    assert!(adapter::handle(
        &ctx,
        "markdown-reference",
        "extension:invoke-action",
        &json!({"actionId":id}),
        &mut s
    )
    .await
    .is_err());
    assert_eq!(committed, s);
}
#[tokio::test]
async fn actual_native_session_routes_errors_retries_search_navigation_and_lifecycle() {
    let ctx = context();
    let sessions = Sessions::default();
    let initial = ipc(&sessions, &ctx, "extension:run-command", launch_body()).await;
    assert_eq!(initial["ok"], true);
    assert_eq!(
        initial["extensionId"],
        "com.tezbar.native.markdown-reference"
    );
    let session = initial["sessionId"].clone();
    assert!(session.as_str().unwrap().starts_with("native-extension:"));
    let view_id = initial["actions"][0]["id"].clone();
    let failure=ipc(&sessions,&ctx,"extension:invoke-action",json!({"sessionId":session,"actionId":"forged","formValues":{"text":"arbitrary","url":"file:///bad"}})).await;
    assert_eq!(failure["ok"], false);
    assert!(failure["message"].as_str().unwrap().contains("stale"));
    let detail = ipc(
        &sessions,
        &ctx,
        "extension:invoke-action",
        json!({"sessionId":session,"actionId":view_id}),
    )
    .await;
    assert_eq!(detail["root"]["type"], "Detail"); // retry remains possible
    let stale = ipc(
        &sessions,
        &ctx,
        "extension:invoke-action",
        json!({"sessionId":session,"actionId":view_id}),
    )
    .await;
    assert_eq!(stale["ok"], false);
    let refresh = ipc(
        &sessions,
        &ctx,
        "extension:refresh-session",
        json!({"sessionId":session}),
    )
    .await;
    assert_eq!(refresh, v2_sdk::unchanged());
    let back_id = detail["root"]["props"]["backActionId"].clone();
    let back = ipc(
        &sessions,
        &ctx,
        "extension:invoke-action",
        json!({"sessionId":session,"actionId":back_id}),
    )
    .await;
    assert_eq!(back["root"]["type"], "List");
    let (old, new) = tokio::join!(
        ipc(
            &sessions,
            &ctx,
            "extension:search-text-changed",
            json!({"sessionId":session,"searchText":"Bold"})
        ),
        ipc(
            &sessions,
            &ctx,
            "extension:search-text-changed",
            json!({"sessionId":session,"searchText":"LI"})
        )
    );
    assert_eq!(new["ok"], true);
    assert_eq!(new["root"]["props"]["searchText"], "LI");
    assert_eq!(new["actions"].as_array().unwrap().len(), 5);
    if old["ok"] == true {
        assert_eq!(
            ipc(
                &sessions,
                &ctx,
                "extension:invoke-action",
                json!({"sessionId":session,"actionId":old["actions"][0]["id"]})
            )
            .await["ok"],
            false
        );
    }
    let current = new["actions"][0]["id"].clone();
    let pushed = ipc(
        &sessions,
        &ctx,
        "extension:invoke-action",
        json!({"sessionId":session,"actionId":current}),
    )
    .await;
    assert_eq!(pushed["ok"], true);
    let restored = ipc(
        &sessions,
        &ctx,
        "extension:invoke-action",
        json!({"sessionId":session,"actionId":pushed["root"]["props"]["backActionId"]}),
    )
    .await;
    assert_eq!(restored["root"]["props"]["searchText"], "LI");
    assert_eq!(
        ipc(
            &sessions,
            &ctx,
            "extension:dispose-session",
            json!({"sessionId":session})
        )
        .await,
        true
    );
    assert_eq!(
        ipc(
            &sessions,
            &ctx,
            "extension:invoke-action",
            json!({"sessionId":session,"actionId":current})
        )
        .await["ok"],
        false
    );
    let fresh = ipc(&sessions, &ctx, "extension:run-command", launch_body()).await;
    assert_ne!(fresh["sessionId"], session);
    assert_eq!(
        ipc(
            &sessions,
            &ctx,
            "extension:invoke-action",
            json!({"sessionId":fresh["sessionId"],"actionId":view_id})
        )
        .await["ok"],
        false
    );
    sessions.clear().await;
    assert_eq!(
        ipc(
            &sessions,
            &ctx,
            "extension:refresh-session",
            json!({"sessionId":fresh["sessionId"]})
        )
        .await["ok"],
        false
    );
    assert_eq!(
        ipc(
            &sessions,
            &ctx,
            "extension:run-command",
            json!({"extensionId":"com.tezbar.native.markdown-reference","commandName":"fabricated"})
        )
        .await["ok"],
        false
    );
}
#[tokio::test]
async fn exact_sdk_routes_preserve_transport_and_negative_envelopes_without_os() {
    let p = std::path::PathBuf::from("/forbidden-test-profile");
    let ctx = CoreContext::new(
        CorePaths {
            data_dir: p.clone(),
            config_dir: p.clone(),
            resource_dir: p.clone(),
            temp_dir: p.clone(),
            home_dir: p,
        },
        None,
    );
    ctx.set_router(Arc::new(|channel, args| {
        Box::pin(async move {
            assert_eq!(args, vec![json!("fixture")]);
            match channel.as_str() {
                "clipboard:write" => Ok(json!({"ok":false,"error":"denied"})),
                "shell:open" => Err("transport failure".into()),
                _ => panic!("Unexpected route"),
            }
        })
    }))
    .unwrap();
    assert_eq!(
        v2_sdk::copy(&ctx, "fixture").await.unwrap(),
        json!({"ok":false,"error":"denied"})
    );
    assert_eq!(
        v2_sdk::open(&ctx, "fixture").await.unwrap_err(),
        "transport failure"
    );
    // This source has no copy/open actions. The adapter never interprets these
    // envelopes or emits success for them; its installed router above forbids all OS calls.
}
