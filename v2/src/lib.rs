//! Offline port of the pinned own V1 source. No host or JavaScript runtime dependency.
use serde_json::{json, Map, Value};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    OnceLock,
};

pub const COMMAND: &str = "markdown-reference";
pub const TITLE: &str = "Search Markdown Syntax";
pub const ID: &str = "com.tezbar.native.markdown-reference";
pub const SOURCE_SHA: &str = "6933819bb52944ac5db4c517f932ffa24a95497e";
static DATA: OnceLock<Value> = OnceLock::new();
// A nonce is consumed even for a subsequently discarded staged search. Never
// derive it from cloned session state, or a cancelled view could become valid.
static VIEW_NONCE: AtomicU64 = AtomicU64::new(1);

pub fn references() -> &'static [Value] {
    DATA.get_or_init(|| {
        serde_json::from_str(include_str!("../data/reference.json"))
            .expect("validated reference data")
    })
    .as_array()
    .expect("reference array")
}

/// Mirrors V1's lowercase, first/second word prefix match, without trimming.
pub fn search(query: &str) -> Vec<usize> {
    let query = query.to_lowercase();
    references()
        .iter()
        .enumerate()
        .filter_map(|(i, reference)| {
            let name = reference["name"]
                .as_str()
                .expect("reference name")
                .to_lowercase();
            (query.is_empty() || name.split(' ').take(2).any(|word| word.starts_with(&query)))
                .then_some(i)
        })
        .collect()
}

/// Retains the original escape library's replacement order and skip labels,
/// including its swapped labels for `#` and `+`. Backticks remain unescaped.
pub fn escape_md(text: &str, skips: &[&str]) -> String {
    const REPLACEMENTS: &[(char, &str, &str)] = &[
        ('*', "\\*", "asterisks"),
        ('#', "\\#", "plus sign"),
        ('+', "\\+", "number signs"),
        ('/', "\\/", "slashes"),
        ('(', "\\(", "parentheses"),
        (')', "\\)", "parentheses"),
        ('[', "\\[", "square brackets"),
        (']', "\\]", "square brackets"),
        ('<', "&lt;", "angle brackets"),
        ('>', "&gt;", "angle brackets"),
        ('_', "\\_", "underscores"),
        ('=', "\\=", "equals sign"),
        ('-', "\\-", "hyphen"),
    ];
    REPLACEMENTS
        .iter()
        .fold(text.to_owned(), |out, (ch, replacement, name)| {
            if skips.contains(name) {
                out
            } else {
                out.replace(*ch, replacement)
            }
        })
}

pub fn detail_markdown(index: usize) -> Option<String> {
    let reference = references().get(index)?;
    let mut out = format!(
        "\n   # {}\n   ---\n   {}\n   \n   ## Examples\n   ---\n  ",
        reference["name"].as_str()?,
        reference["description"].as_str()?
    );
    for example in reference["examples"].as_array()? {
        out.push_str(&format!(
            "\n{}\n",
            escape_md(example["markdown"].as_str()?, &[])
        ));
    }
    let additional = reference["additional_examples"].as_array()?;
    if !additional.is_empty() {
        out.push_str(" ## Additional Examples");
        for example in additional {
            out.push_str(&format!(
                "\n ## {}\n{}\n\n{}\n",
                example["name"].as_str()?,
                example["description"].as_str()?,
                escape_md(example["markdown"].as_str()?, &[])
            ));
        }
    }
    Some(out)
}

#[derive(Debug, PartialEq)]
pub enum Output {
    View { root: Value, actions: Vec<Value> },
    Unchanged,
    Disposed(bool),
}

fn render(state: &mut Value) -> Result<Output, String> {
    let nonce = VIEW_NONCE
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
        .map_err(|_| "Action ID space exhausted; restart the application")?;
    let mut allowed = Map::new();
    let mut actions = Vec::new();
    let root = if let Some(index) = state["detail"].as_u64() {
        let index = usize::try_from(index).map_err(|_| "Invalid reference index")?;
        let markdown = detail_markdown(index).ok_or("Invalid reference index")?;
        let id = format!("markdown-reference:{nonce}:back");
        allowed.insert(id.clone(), json!({"kind":"back"}));
        actions.push(json!({"id":id,"title":"Back","kind":"pop"}));
        json!({"type":"Detail","props":{"markdown":markdown,"backActionId":id}})
    } else {
        let query = state["query"].as_str().ok_or("Invalid search state")?;
        let children: Vec<_> = search(query).into_iter().map(|index| {
            let id = format!("markdown-reference:{nonce}:view:{index}");
            allowed.insert(id.clone(), json!({"kind":"view","index":index}));
            actions.push(json!({"id":id,"title":"View","kind":"push"}));
            json!({"type":"List.Item","props":{"id":format!("reference:{index}"),"title":references()[index]["name"],"actionIds":[id]}})
        }).collect();
        json!({"type":"List","props":{"navigationTitle":TITLE,"searchBarPlaceholder":TITLE,"searchText":query,"__hasServerSearch":true},"children":children})
    };
    state["actions"] = Value::Object(allowed);
    Ok(Output::View { root, actions })
}

/// Call with the host-owned state. Search mutations belong in native.rs's
/// staged value; heartbeat refreshes never mint or publish new capabilities.
pub fn handle(
    command: &str,
    operation: &str,
    payload: &Value,
    state: &mut Value,
) -> Result<Output, String> {
    if command != COMMAND {
        return Err("Unknown Markdown Reference command".into());
    }
    if operation == "extension:run-command" {
        let mut next = json!({"active":true,"query":"","detail":null});
        let output = render(&mut next)?;
        *state = next;
        return Ok(output);
    }
    if operation == "extension:dispose-session" {
        let active = state["active"] == true;
        *state = json!({"active":false});
        return Ok(Output::Disposed(active));
    }
    if state["active"] != true {
        return Err("Markdown Reference session is not active".into());
    }
    match operation {
        "extension:refresh-session" | "extension:load-more" => Ok(Output::Unchanged),
        "extension:search-text-changed" => {
            if !state["detail"].is_null() {
                return Err("Search is only available on the reference list".into());
            }
            let query = payload["searchText"].as_str().ok_or("Missing searchText")?;
            let mut next = state.clone();
            next["query"] = json!(query);
            let output = render(&mut next)?;
            *state = next;
            Ok(output)
        }
        "extension:invoke-action" => {
            let id = payload["actionId"].as_str().ok_or("Missing actionId")?;
            let action = state["actions"]
                .get(id)
                .ok_or("Unknown or stale Markdown Reference action")?;
            let mut next = state.clone();
            match action["kind"].as_str() {
                Some("view") if state["detail"].is_null() => {
                    let index = action["index"].as_u64().ok_or("Invalid reference action")?;
                    next["detail"] = json!(index);
                }
                Some("back") if state["detail"].is_u64() => next["detail"] = Value::Null,
                _ => return Err("Action does not belong to this screen".into()),
            }
            let output = render(&mut next)?;
            *state = next;
            Ok(output)
        }
        _ => Err(format!(
            "Unsupported Markdown Reference operation: {operation}"
        )),
    }
}
