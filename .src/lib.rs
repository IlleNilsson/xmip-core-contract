// deny, not forbid: `export` stands on the far side of the C boundary and
// allows it there alone, each block with a SAFETY comment.
#![deny(unsafe_code)]

// What every technology of this capability shares, held here rather than
// copied into each (ADR-0044): the `$ref` reading, the place an issue names
// (JSON Pointer, dotted, XPath), the layout types, the EDI
// segment and the test fixture that builds a Stream. The byte cursor and the
// varint are codec's (xmip-core-library-codec) since 2026-09-24. The trait
// a contract implements, and the export that makes a Rust contract a module a
// node loads through the C ABI, are this capability's own: a provider's
// contract implements the same trait core's do.
pub mod export;
#[cfg(feature = "test-support")]
pub mod fixture;
pub mod layout;
pub mod place;
pub mod reference;
pub mod segment;

use std::borrow::Cow;
use stream::Stream;
use xcore::settings::{Applies, Given, Read, Settings};

/// A contract's name, as a Location's configuration refers to it: `csv`,
/// `json-schema`, a provider's own.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ContractId(pub String);

/// What a contract is: its name, the version of its rules, and the media type
/// of the content it holds a Stream to (`text/csv`, `application/protobuf`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractDescriptor {
    pub id: ContractId,
    pub version: String,
    pub representation: String,
}

/// One way a Stream departs from a contract: a short `code` a surface can
/// count by, a `message` a person reads, and where in the content, when the
/// contract can say. A code is nearly always a word the technology names in
/// its source, so it is borrowed, not allocated, per issue.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationIssue {
    pub code: Cow<'static, str>,
    pub message: String,
    pub path: Option<String>,
}

impl ValidationIssue {
    /// An issue with `code` and `message`, at `path` when there is one.
    #[must_use]
    pub fn new(
        code: impl Into<Cow<'static, str>>,
        message: impl Into<String>,
        path: Option<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            path,
        }
    }

    /// An issue at a place the technology can name.
    #[must_use]
    pub fn at(
        code: impl Into<Cow<'static, str>>,
        message: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self::new(code, message, Some(path.into()))
    }

    /// The one issue every technology raises alike: the Stream cannot be read
    /// as the representation at all, so no path can be named.
    #[must_use]
    pub fn malformed(message: impl Into<String>) -> Self {
        Self::new("malformed", message, None)
    }
}

/// What validating a Stream concluded: valid exactly when there is no issue.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationResult {
    pub valid: bool,
    pub issues: Vec<ValidationIssue>,
}

impl ValidationResult {
    /// Valid exactly when there is no issue.
    #[must_use]
    pub fn of(issues: Vec<ValidationIssue>) -> Self {
        Self {
            valid: issues.is_empty(),
            issues,
        }
    }
}

// A contract that could not judge at all — the bytes are not the kind of
// thing it reads — as against one that judged and found issues.
xcore::declare_error!(ContractError);

/// The rules a Stream is held to before Xmip accepts it (ADR-0010).
///
/// A departure the contract can describe is a [`ValidationIssue`] in an
/// `Ok` result; an `Err` says the contract could not judge the Stream at all.
pub trait Contract: Send + Sync {
    /// What this contract is.
    fn descriptor(&self) -> &ContractDescriptor;

    /// Whether `stream` looks like this contract's representation — a media
    /// type, a magic number, a first line — cheaply, without validating it.
    ///
    /// # Errors
    /// The Stream cannot be looked at as this representation at all.
    fn identify(&self, stream: &Stream) -> Result<bool, ContractError>;

    /// Hold `stream` to the rules, and say every way it departs from them.
    ///
    /// # Errors
    /// The contract cannot judge the Stream at all; a departure it can
    /// describe is an issue in the result, not an error.
    fn validate(&self, stream: &Stream) -> Result<ValidationResult, ContractError>;
}

/// A technology that makes a contract from a reference — a schema file, a
/// `.proto`, an XSD — named in a Location's configuration.
pub trait ContractFactory: Send + Sync {
    /// The technology's name, as the configuration names it.
    fn technology(&self) -> &'static str;

    /// Every setting a Location gives this technology, declared once in its
    /// own crate (ADR-0064, amendment 2026-09-26): `reference` where it takes
    /// one, anything else it reads, and an empty list when it takes nothing.
    /// `technology` is its module name, `env!("CARGO_PKG_NAME")`. The shape
    /// is `xcore::settings`, the one a transport's settings take too.
    fn settings(&self) -> &'static Settings;

    /// The contract `reference` describes.
    ///
    /// # Errors
    /// The reference cannot be read, or does not describe a contract.
    fn load(&self, reference: &str) -> Result<Box<dyn Contract>, ContractError>;

    /// The contract a Location's settings describe, as the declaration read
    /// them. Unless the technology reads more, that is its `reference`, or
    /// none, handed to [`ContractFactory::load`] — the one string the C
    /// ABI's `load` carries.
    ///
    /// # Errors
    /// As [`ContractFactory::load`].
    fn configured(&self, settings: &Read) -> Result<Box<dyn Contract>, ContractError> {
        self.load(settings.optional_text("reference").unwrap_or(""))
    }

    /// Read what a Location on `side` gave through
    /// [`ContractFactory::settings`], and build the contract from it: the one
    /// way from a Location's table to a contract.
    ///
    /// # Errors
    /// Every setting the declaration refuses, each naming the technology and
    /// the setting; then whatever [`ContractFactory::configured`] refuses.
    fn open(
        &self,
        side: Applies,
        given: &[(String, Given)],
    ) -> Result<Box<dyn Contract>, ContractError> {
        let read = self
            .settings()
            .read(side, given)
            .map_err(|refused| ContractError::new(refused.to_string()))?;
        self.configured(&read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xcore::StreamId;

    /// The smallest contract there is: text, held when it is UTF-8.
    struct Text(ContractDescriptor);

    impl Contract for Text {
        fn descriptor(&self) -> &ContractDescriptor {
            &self.0
        }

        fn identify(&self, stream: &Stream) -> Result<bool, ContractError> {
            Ok(stream.media_type() == Some("text/plain"))
        }

        fn validate(&self, stream: &Stream) -> Result<ValidationResult, ContractError> {
            let issues = match stream.text() {
                Ok(_) => Vec::new(),
                Err(error) => vec![ValidationIssue {
                    code: "not-text".into(),
                    message: error.to_string(),
                    path: Some(format!("byte {}", error.valid_up_to())),
                }],
            };
            Ok(ValidationResult {
                valid: issues.is_empty(),
                issues,
            })
        }
    }

    struct Factory;

    impl ContractFactory for Factory {
        fn technology(&self) -> &'static str {
            "text"
        }

        fn settings(&self) -> &'static Settings {
            const NONE: &Settings = &Settings::none("xmip-core-contract-text");
            NONE
        }

        fn load(&self, reference: &str) -> Result<Box<dyn Contract>, ContractError> {
            if reference.is_empty() {
                Ok(Box::new(Text(descriptor())))
            } else {
                Err(ContractError {
                    message: format!("text takes no reference, got {reference:?}"),
                })
            }
        }
    }

    fn descriptor() -> ContractDescriptor {
        ContractDescriptor {
            id: ContractId("text".to_string()),
            version: "1".to_string(),
            representation: "text/plain".to_string(),
        }
    }

    fn stream(bytes: &[u8], media: Option<&str>) -> Stream {
        Stream::new(StreamId::new(1), bytes.to_vec(), media.map(str::to_string))
    }

    #[test]
    fn a_contract_identifies_by_representation_and_validates_the_bytes() {
        let contract = Factory.load("").expect("loaded");
        assert_eq!(contract.descriptor(), &descriptor());
        assert!(
            contract
                .identify(&stream(b"x", Some("text/plain")))
                .expect("identified")
        );
        assert!(
            !contract
                .identify(&stream(b"x", Some("application/json")))
                .expect("identified")
        );
        assert!(
            contract
                .validate(&stream(b"plain", None))
                .expect("validated")
                .valid
        );
        let held = contract
            .validate(&stream(&[0xff, 0xfe], None))
            .expect("validated");
        assert!(!held.valid);
        assert_eq!(held.issues[0].code, "not-text");
        assert_eq!(held.issues[0].path.as_deref(), Some("byte 0"));
    }

    #[test]
    fn a_factory_names_its_technology_and_refuses_what_it_cannot_load() {
        assert_eq!(Factory.technology(), "text");
        let refused = Factory.load("schema.xsd").err().expect("refused");
        assert!(refused.to_string().contains("schema.xsd"));
        assert_eq!(ContractError::new("why").to_string(), "why");
    }

    #[test]
    fn an_issue_is_built_three_ways_and_a_result_is_valid_without_one() {
        let placed = ValidationIssue::at("structure", "paths is missing", "paths");
        assert_eq!(
            placed,
            ValidationIssue::new("structure", "paths is missing", Some("paths".into()))
        );
        let malformed = ValidationIssue::malformed("not JSON");
        assert_eq!(malformed.code, "malformed");
        assert_eq!(malformed.path, None);
        assert!(ValidationResult::of(Vec::new()).valid);
        let held = ValidationResult::of(vec![malformed]);
        assert!(!held.valid);
        assert_eq!(held.issues.len(), 1);
    }
}
