#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use xmip_stream::Stream;

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationResult {
    pub valid: bool,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug)]
pub struct ContractError {
    pub message: String,
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for ContractError {}

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

#[derive(Clone, Debug, PartialEq)]
pub enum StructuredValue {
    Null,
    Bool(bool),
    Integer(i64),
    Decimal(f64),
    Text(String),
    Binary(Vec<u8>),
}
