use markdown_reference_beta_harness::native_core::{
    common::{CoreContext, CorePaths},
    extensions::v2_markdown_reference::handle,
};
use serde_json::json;
#[tokio::main]
async fn main() {
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
    let mut state = json!({});
    let list = handle(
        &ctx,
        "markdown-reference",
        "extension:run-command",
        &json!({}),
        &mut state,
    )
    .await
    .unwrap();
    let search = handle(
        &ctx,
        "markdown-reference",
        "extension:search-text-changed",
        &json!({"searchText":"LI"}),
        &mut state,
    )
    .await
    .unwrap();
    let detail = handle(
        &ctx,
        "markdown-reference",
        "extension:invoke-action",
        &json!({"actionId":search["actions"][0]["id"]}),
        &mut state,
    )
    .await
    .unwrap();
    let back = handle(
        &ctx,
        "markdown-reference",
        "extension:invoke-action",
        &json!({"actionId":detail["root"]["props"]["backActionId"]}),
        &mut state,
    )
    .await
    .unwrap();
    let mut topics = Vec::new();
    for index in 0..15 {
        let list = handle(
            &ctx,
            "markdown-reference",
            "extension:run-command",
            &json!({}),
            &mut state,
        )
        .await
        .unwrap();
        topics.push(
            handle(
                &ctx,
                "markdown-reference",
                "extension:invoke-action",
                &json!({"actionId":list["actions"][index]["id"]}),
                &mut state,
            )
            .await
            .unwrap(),
        );
    }
    println!(
        "{}",
        json!({"list":list,"search":search,"detail":detail,"back":back,"topics":topics})
    );
}
