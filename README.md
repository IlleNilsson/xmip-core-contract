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

Where a contract is called today: the runtime's loader configures, starts
and stops a contract table it opened
([built, not in the assembled service](../../../../doc/architecture/estate-map.md#module-loading)),
and the Playground validates its generated content through the contract
technologies it links. A node holding a Location's Streams to a contract is
[decided, not built](../../../../doc/architecture/estate-map.md#arrival-validation):
a node refuses to start a Location that names one, and a new contract
reaches a node only by being linked into `xmip-service` and built again
with `Build-XmipService`.

What a Location gives a contract is declared by the contract itself:
`ContractFactory::settings`, in the shape a transport's settings take
(`xcore::settings`), read through by `open` (ADR-0064, amendment
2026-09-26).

ADR-0010 and ADR-0042 govern it; each format is a technology mounted under
this repository, `csv` the reference, and `doc/adding-a-contract.md` beside
this file says how one is added. `architecture.toml` names them.
