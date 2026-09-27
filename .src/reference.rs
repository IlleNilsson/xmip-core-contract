//! The `$ref` a JSON description makes, read one way for every technology
//! that reads one (ADR-0044): a reference that begins with `#` is a URI
//! fragment holding a JSON Pointer (RFC 6901 section 6), percent-decoded
//! first, then resolved in the document itself. The `AsyncAPI` and `OpenAPI`
//! technologies hold every such reference to landing; the JSON Schema
//! technology resolves its schemas through the same [`resolve`].

use crate::ValidationIssue;
use crate::place::Place;
use serde_json::Value;

/// What `reference` points at inside `document`, when it is a fragment
/// (`#`, `#/$defs/id`, `#/paths/~1orders%7Bid%7D`) and lands; `None` for a
/// reference to another document or one that does not land.
#[must_use]
pub fn resolve<'a>(document: &'a Value, reference: &str) -> Option<&'a Value> {
    let fragment = reference.strip_prefix('#')?;
    if fragment.contains('%') {
        document.pointer(&net::percent::decode(fragment))
    } else {
        document.pointer(fragment)
    }
}

/// Every `$ref` under `document` that begins with `#` and does not land, each
/// coded `reference` at the JSON Pointer of the object that holds it.
#[must_use]
pub fn dangling(document: &Value) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    walk(document, document, &Place::Root, &mut issues);
    issues
}

fn walk(root: &Value, value: &Value, place: &Place<'_>, issues: &mut Vec<ValidationIssue>) {
    match value {
        Value::Object(object) => {
            if let Some(Value::String(target)) = object.get("$ref")
                && target.starts_with('#')
                && resolve(root, target).is_none()
            {
                issues.push(ValidationIssue::at(
                    "reference",
                    format!("$ref {target} does not land"),
                    place.pointer(),
                ));
            }
            for (key, child) in object {
                walk(root, child, &place.field(key), issues);
            }
        }
        Value::Array(items) => {
            for (i, child) in items.iter().enumerate() {
                walk(root, child, &place.index(i), issues);
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
        assert_eq!(issues[0].path.as_deref(), Some("/b/0"));
        assert!(dangling(&Value::Null).is_empty());
    }

    #[test]
    fn a_fragment_is_percent_decoded_before_it_is_a_pointer() {
        let document: Value =
            serde_json::from_str(r#"{"paths": {"/orders/{id}": {"x": 1}}, "a b": 2, "%": 3}"#)
                .expect("json");
        assert_eq!(
            resolve(&document, "#/paths/~1orders~1%7Bid%7D/x"),
            Some(&Value::from(1))
        );
        assert_eq!(resolve(&document, "#/a%20b"), Some(&Value::from(2)));
        assert_eq!(resolve(&document, "#/%25"), Some(&Value::from(3)));
        assert_eq!(resolve(&document, "#"), Some(&document));
        assert_eq!(resolve(&document, "other.json#/a"), None);
    }
}
