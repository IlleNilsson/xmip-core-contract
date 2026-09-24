//! The `$ref` claim a JSON description makes, shared by the `AsyncAPI` and
//! `OpenAPI` technologies (ADR-0044): every reference that begins with `#`
//! must land in the document itself. The pointer after the `#` is RFC 6901,
//! which `serde_json` already resolves.

use sdk::contract::ValidationIssue;
use serde_json::Value;

/// Every `$ref` under `document` that begins with `#` and does not land, each
/// coded `reference` at the dotted path of the object that holds it.
#[must_use]
pub fn dangling(document: &Value) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    walk(document, document, "", &mut issues);
    issues
}

fn walk(root: &Value, value: &Value, path: &str, issues: &mut Vec<ValidationIssue>) {
    match value {
        Value::Object(object) => {
            if let Some(Value::String(target)) = object.get("$ref")
                && let Some(pointer) = target.strip_prefix('#')
                && root.pointer(pointer).is_none()
            {
                issues.push(ValidationIssue::at(
                    "reference",
                    &format!("$ref {target} does not land"),
                    path.trim_start_matches('.'),
                ));
            }
            for (key, child) in object {
                walk(root, child, &format!("{path}.{key}"), issues);
            }
        }
        Value::Array(items) => {
            for (i, child) in items.iter().enumerate() {
                walk(root, child, &format!("{path}[{i}]"), issues);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reference_that_lands_is_silent_and_one_that_does_not_is_placed() {
        let document: Value = serde_json::from_str(
            r##"{"a": {"$ref": "#/b/0"}, "b": [{"$ref": "#/nowhere"}],
                 "c": {"$ref": "https://example.test/other#/x"}}"##,
        )
        .expect("json");
        let issues = dangling(&document);
        assert_eq!(issues.len(), 1, "{issues:?}");
        assert_eq!(issues[0].code, "reference");
        assert_eq!(issues[0].message, "$ref #/nowhere does not land");
        assert_eq!(issues[0].path.as_deref(), Some("b[0]"));
        assert!(dangling(&Value::Null).is_empty());
    }
}
