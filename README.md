# xmip-core-contract

The Contract: what a Stream must be for Xmip to accept it as a Message. A
`Contract` identifies whether a Stream is its format and validates that it is
well-formed, answering with issues an operator can read; what every format
technology shares — issue and result constructors, the reference walk, the
layout types, the EDI segment — lives here rather than in each (ADR-0044).

A Contract holds well-formedness always and conformance when named
(ADR-0042). It does not parse for its own sake, does not serialize and does
not execute a Path; the representation technologies materialize, Paths
address, and this evaluates (`repository-model.md` section 5).

ADR-0010 and ADR-0042 govern it; each format is a technology mounted under
this repository, `csv` the reference, and `doc/adding-a-contract.md` beside
this file says how one is added. `architecture.toml` names them.
