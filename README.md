# xmip-core-contract

The Contract: what a Stream must be for Xmip to accept it as a Message. A
`Contract` identifies whether a Stream is its format and validates that it is
well-formed, answering with issues an operator can read; what every format
technology shares — issue and result constructors, the `$ref` reading, the
place an issue names, the layout types, the EDI segment — lives here rather
than in each (ADR-0044).

An issue's `code` is borrowed, not allocated, and its place is spelled only
when an issue is raised: a technology walks its content with a `Place`, one
step per level, and writes it in its representation's form — a JSON document
as a JSON Pointer (RFC 6901, the empty pointer for the whole document), a
binary datum by field names (`order.lines[2]`), an XML document as an XPath
(`/order/line[2]/@currency`). A `$ref` is a URI fragment, percent-decoded by
`xmip-core-library-net` before it is a JSON Pointer, read one way by the
`OpenAPI`, `AsyncAPI` and JSON Schema technologies. A contract reads a Stream as
text through `Stream::text`, decoded once and shared by `identify` and
`validate`.

A Contract holds well-formedness always and conformance when named
(ADR-0042). It does not parse for its own sake, does not serialize and does
not execute a Path; the representation technologies materialize, Paths
address, and this evaluates (`repository-model.md` section 5).

What a Location gives a contract is declared by the contract itself:
`ContractFactory::settings`, in the shape a transport's settings take
(`xcore::settings`), read through by `open` (ADR-0064, amendment
2026-09-26).

ADR-0010 and ADR-0042 govern it; each format is a technology mounted under
this repository, `csv` the reference, and `doc/adding-a-contract.md` beside
this file says how one is added. `architecture.toml` names them.
