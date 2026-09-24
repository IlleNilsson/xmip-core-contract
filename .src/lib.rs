#![forbid(unsafe_code)]

// What every technology of this capability shares, held here rather than
// copied into each (ADR-0044): the `$ref` walk, the varint cursor, the layout
// types, the EDI segment and the test fixture that builds a Stream. The trait a
// contract implements, and the types it answers in, are the SDK's
// (`sdk::contract`, ADR-0061): a provider builds against them there, and
// core's contracts implement the same ones.
#[cfg(feature = "test-support")]
pub mod fixture;
pub mod layout;
pub mod reference;
pub mod segment;
pub mod varint;
