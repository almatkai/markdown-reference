use serde_json::{json, Value};
use tezbar_markdown_reference::{
    detail_markdown, escape_md, handle, references, search, Output, COMMAND,
};
fn oracle() -> Value {
    serde_json::from_str(include_str!("oracle.json")).unwrap()
}
fn call(op: &str, payload: Value, state: &mut Value) -> Output {
    handle(COMMAND, op, &payload, state).unwrap()
}
fn view(out: Output) -> (Value, Vec<Value>) {
    match out {
        Output::View { root, actions } => (root, actions),
        _ => panic!("Expected view"),
    }
}
fn run(s: &mut Value) -> (Value, Vec<Value>) {
    view(call("extension:run-command", json!({}), s))
}
fn invoke(id: &Value, s: &mut Value) -> (Value, Vec<Value>) {
    view(call(
        "extension:invoke-action",
        json!({"actionId":id,"formValues":{"index":999,"url":"file:///forged","text":"forged"}}),
        s,
    ))
}
fn rejected(id: &Value, s: &mut Value) {
    let before = s.clone();
    assert!(handle(
        COMMAND,
        "extension:invoke-action",
        &json!({"actionId":id}),
        s
    )
    .is_err());
    assert_eq!(*s, before);
}
#[test]
fn all_own_source_data_search_escape_and_detail_bytes_match_executable_oracle() {
    let o = oracle();
    assert_eq!(references(), o["references"].as_array().unwrap());
    for c in o["searches"].as_array().unwrap() {
        let actual: Vec<_> = search(c["query"].as_str().unwrap())
            .into_iter()
            .map(|i| references()[i]["name"].clone())
            .collect();
        assert_eq!(json!(actual), c["names"], "query {:?}", c["query"]);
    }
    for (i, md) in o["markdowns"].as_array().unwrap().iter().enumerate() {
        assert_eq!(detail_markdown(i).unwrap(), md.as_str().unwrap());
    }
    assert!(detail_markdown(usize::MAX).is_none());
    for c in o["escapeCases"].as_array().unwrap() {
        let skips: Vec<_> = c["skips"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap())
            .collect();
        assert_eq!(
            escape_md(c["text"].as_str().unwrap(), &skips),
            c["expected"]
        );
    }
}
#[test]
fn every_visible_action_and_back_preserves_draft_and_content() {
    for c in oracle()["searches"].as_array().unwrap() {
        let mut state = json!({});
        run(&mut state);
        let (root, actions) = view(call(
            "extension:search-text-changed",
            json!({"searchText":c["query"]}),
            &mut state,
        ));
        assert_eq!(root["props"]["searchText"], c["query"]);
        assert_eq!(root["props"]["__hasServerSearch"], true);
        let expected: Vec<_> = root["children"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["props"]["title"].clone())
            .collect();
        assert_eq!(json!(expected), c["names"]);
        for action_index in 0..actions.len() {
            let (_, current) = view(call(
                "extension:search-text-changed",
                json!({"searchText":c["query"]}),
                &mut state,
            ));
            let old = current[action_index]["id"].clone();
            let (detail, back) = invoke(&old, &mut state);
            assert_eq!(detail["type"], "Detail");
            assert_eq!(back.len(), 1);
            assert_eq!(back[0]["title"], "Back");
            assert_eq!(detail["props"]["backActionId"], back[0]["id"]);
            let index = references()
                .iter()
                .position(|r| r["name"] == c["names"][action_index])
                .unwrap();
            assert_eq!(detail["props"]["markdown"], detail_markdown(index).unwrap());
            rejected(&old, &mut state);
            let before = state.clone();
            assert!(handle(
                COMMAND,
                "extension:search-text-changed",
                &json!({"searchText":"changed behind detail"}),
                &mut state
            )
            .is_err());
            assert_eq!(before, state);
            let (list, _) = invoke(&back[0]["id"], &mut state);
            assert_eq!(list["props"]["searchText"], c["query"]);
            rejected(&back[0]["id"], &mut state);
        }
    }
}
#[test]
fn staged_queries_heartbeats_disposal_relaunch_and_forgery_are_safe() {
    let mut committed = json!({});
    let (_, a) = run(&mut committed);
    let initial = a[0]["id"].clone();
    let mut staged = committed.clone();
    let (_, discarded) = view(call(
        "extension:search-text-changed",
        json!({"searchText":"bold"}),
        &mut staged,
    ));
    rejected(&discarded[0]["id"], &mut committed); // cancelled native.rs staged search
    let before = committed.clone();
    let mut refresh_clone = committed.clone();
    assert_eq!(
        call("extension:refresh-session", json!({}), &mut refresh_clone),
        Output::Unchanged
    );
    assert_eq!(before, refresh_clone);
    assert_eq!(
        call("extension:load-more", json!({}), &mut committed),
        Output::Unchanged
    );
    assert_eq!(before, committed);
    for payload in [
        json!({}),
        json!({"actionId":null}),
        json!({"actionId":17}),
        json!({"actionId":"__nav_pop__"}),
        json!({"actionId":"copy"}),
        json!({"actionId":"markdown-reference:1:view:999"}),
        json!({"actionId":"file:///x","text":"arbitrary","url":"https://attacker.invalid"}),
    ] {
        assert!(handle(COMMAND, "extension:invoke-action", &payload, &mut committed).is_err());
        assert_eq!(committed, before);
    }
    invoke(&initial, &mut committed); // previous failures leave the legitimate action retryable
    let (_, a2) = run(&mut committed);
    rejected(&initial, &mut committed);
    let (_, after_query) = view(call(
        "extension:search-text-changed",
        json!({"searchText":"bold"}),
        &mut committed,
    ));
    rejected(&a2[0]["id"], &mut committed);
    rejected(&discarded[0]["id"], &mut committed);
    let (_, same) = view(call(
        "extension:search-text-changed",
        json!({"searchText":"bold"}),
        &mut committed,
    ));
    rejected(&after_query[0]["id"], &mut committed);
    let mut other = json!({});
    run(&mut other);
    rejected(&same[0]["id"], &mut other);
    assert_eq!(
        call("extension:dispose-session", json!({}), &mut committed),
        Output::Disposed(true)
    );
    rejected(&same[0]["id"], &mut committed);
    assert_eq!(
        call("extension:dispose-session", json!({}), &mut committed),
        Output::Disposed(false)
    );
    run(&mut committed);
    rejected(&same[0]["id"], &mut committed);
    let before = committed.clone();
    assert!(handle(
        "unknown",
        "extension:run-command",
        &json!({}),
        &mut committed
    )
    .is_err());
    assert!(handle(COMMAND, "unsupported", &json!({}), &mut committed).is_err());
    assert!(handle(
        COMMAND,
        "extension:search-text-changed",
        &json!({"searchText":17}),
        &mut committed
    )
    .is_err());
    assert_eq!(before, committed);
}
