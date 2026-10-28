//! Validate, the standard unit that holds a Stream to its format and, where
//! a step names one, to a contract, as data (ADR-0066, amendment
//! 2026-10-22; the owner: *Validation according to format or custom
//! contracts has to be in there*). Its `format` names a technology of this
//! capability, which the module linking the technologies opens it through;
//! whether a step validates at all stays the artifact's `validate`
//! (ADR-0031, amendment 2026-10-05).

use node::Stage;
use node::unit::Declaration;
use xcore::settings::{Applies, Kind, Presence, Setting, Settings};

/// Validate's name, as a step names it.
pub const VALIDATE: &str = "Validate";

/// Validate, as data.
pub const DECLARATION: &Declaration = &Declaration {
    name: VALIDATE,
    capability: "validate",
    stages: &Stage::ALL,
    settings: &Settings {
        technology: VALIDATE,
        settings: &[
            Setting {
                name: "format",
                kind: Kind::Text,
                presence: Presence::Required,
                meaning: "The contract technology the Stream is held to, by its name: xml-schema, json-schema, csv.",
                applies: Applies::Both,
            },
            Setting {
                name: "contract",
                kind: Kind::Text,
                presence: Presence::Optional,
                meaning: "The contract held to beside well-formedness: a schema's file, a message type.",
                applies: Applies::Both,
            },
        ],
    },
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_is_declared_soundly() {
        assert!(
            DECLARATION.problems().is_empty(),
            "{:?}",
            DECLARATION.problems()
        );
    }
}
