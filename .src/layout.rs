//! What a layout asks of a document — the keys it requires and the seven
//! types a key may be — shared by the TOML and YAML technologies (ADR-0044),
//! whose layouts are one language written in two notations. Whether a
//! notation's value is of a kind, and what a notation calls a value, stay
//! with the notation.

use crate::{ContractError, ValidationIssue};

/// The seven types a layout may ask for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    String,
    Integer,
    Float,
    Boolean,
    Table,
    Array,
    Datetime,
}

impl Kind {
    /// The type named in a layout, if the name is one of the seven.
    #[must_use]
    pub fn named(name: &str) -> Option<Self> {
        Some(match name {
            "string" => Self::String,
            "integer" => Self::Integer,
            "float" => Self::Float,
            "boolean" => Self::Boolean,
            "table" => Self::Table,
            "array" => Self::Array,
            "datetime" => Self::Datetime,
            _ => return None,
        })
    }

    /// The refusal when a leaf at `path` names a type that is not one of the
    /// seven — at bind time, not when a Stream arrives (ADR-0042).
    #[must_use]
    pub fn unknown(path: &str, name: &str) -> ContractError {
        ContractError::new(format!(
            "{path} asks for {name:?}; a layout type is string, integer, float, boolean, \
             table, array or datetime"
        ))
    }

    /// The refusal when a leaf at `path` is not a type name at all but a
    /// value the notation calls `actual`.
    #[must_use]
    pub fn not_a_name(path: &str, actual: &str) -> ContractError {
        ContractError::new(format!(
            "{path} is {actual}; a layout leaf names a type as a string"
        ))
    }
}

/// One key the layout requires: its dotted path and its type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Required {
    pub path: String,
    pub kind: Kind,
}

impl Required {
    /// The issue when the document has no such key.
    #[must_use]
    pub fn missing(&self) -> ValidationIssue {
        ValidationIssue::at(
            "required",
            &format!("{} is required", self.path),
            &self.path,
        )
    }

    /// The issue when the key holds a value of another type, `actual` being
    /// the notation's own name for what it holds.
    #[must_use]
    pub fn mismatched(&self, actual: &str) -> ValidationIssue {
        let message = format!(
            "{} is {actual}, the layout asks for {:?}",
            self.path, self.kind
        )
        .to_lowercase();
        ValidationIssue::at("type", &message, &self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_seven_names_are_known_and_nothing_else_is() {
        assert_eq!(Kind::named("string"), Some(Kind::String));
        assert_eq!(Kind::named("datetime"), Some(Kind::Datetime));
        assert_eq!(Kind::named("number"), None);
        assert!(
            Kind::unknown("port", "number")
                .message
                .contains("port asks for \"number\"")
        );
        assert!(
            Kind::not_a_name("port", "integer")
                .message
                .contains("port is integer")
        );
    }

    #[test]
    fn a_required_key_names_its_path_in_both_issues() {
        let required = Required {
            path: "service.name".to_string(),
            kind: Kind::String,
        };
        let missing = required.missing();
        assert_eq!(missing.code, "required");
        assert_eq!(missing.path.as_deref(), Some("service.name"));
        let mismatched = required.mismatched("integer");
        assert_eq!(mismatched.code, "type");
        assert_eq!(
            mismatched.message,
            "service.name is integer, the layout asks for string"
        );
    }
}
