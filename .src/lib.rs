#![forbid(unsafe_code)]

// What every technology of this capability shares, held here rather than
// copied into each (ADR-0044): the issue and result constructors, the `$ref`
// walk, the varint cursor, the layout types, the EDI segment and the test
// fixture that builds a Stream.
#[cfg(feature = "test-support")]
pub mod fixture;
pub mod layout;
pub mod reference;
pub mod segment;
pub mod varint;

use stream::Stream;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ContractId(pub String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractDescriptor {
    pub id: ContractId,
    pub version: String,
    pub representation: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationIssue {
    pub code: String,
    pub message: String,
    pub path: Option<String>,
}

impl ValidationIssue {
    /// An issue with `code` and `message`, at `path` when there is one.
    #[must_use]
    pub fn new(code: &str, message: &str, path: Option<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            path,
        }
    }

    /// An issue at a place the technology can name.
    #[must_use]
    pub fn at(code: &str, message: &str, path: &str) -> Self {
        Self::new(code, message, Some(path.to_string()))
    }

    /// The one issue every technology raises alike: the Stream cannot be read
    /// as the representation at all, so no path can be named.
    #[must_use]
    pub fn malformed(message: &str) -> Self {
        Self::new("malformed", message, None)
    }
}

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

xcore::declare_error!(ContractError);

pub trait Contract: Send + Sync {
    fn descriptor(&self) -> &ContractDescriptor;
    fn identify(&self, stream: &Stream) -> Result<bool, ContractError>;
    fn validate(&self, stream: &Stream) -> Result<ValidationResult, ContractError>;
}

pub trait ContractFactory: Send + Sync {
    fn technology(&self) -> &'static str;
    fn load(&self, reference: &str) -> Result<Box<dyn Contract>, ContractError>;
}

pub trait StructureReader: Send + Sync {
    fn contract(&self) -> &ContractDescriptor;
    fn read(&self, path: &str) -> Result<Option<StructuredValue>, ContractError>;
}

pub trait StructureWriter: Send {
    fn contract(&self) -> &ContractDescriptor;
    fn write(&mut self, path: &str, value: StructuredValue) -> Result<(), ContractError>;
    fn finish(self: Box<Self>) -> Result<Stream, ContractError>;
}

/// A structured content field's value is one scalar, the shared `ScalarValue`
/// primitive (foundation/core) — `StructuredValue` is contract's name for it.
/// Because `context::ContextValue` aliases the same type, promoting a field into
/// a property needs no conversion: they are one type, not two identical ones.
pub use xcore::ScalarValue as StructuredValue;

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
            let issues = match std::str::from_utf8(stream.bytes()) {
                Ok(_) => Vec::new(),
                Err(error) => vec![ValidationIssue {
                    code: "not-text".to_string(),
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
        assert_eq!(StructuredValue::Integer(1), xcore::ScalarValue::Integer(1));
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
