# Adding a contract

Moved here from the estate root on 2026-09-12 (ADR-0020 clause 3: the document lives where its subject lives).


Identical shape to a transport, against a different base. **`csv` is the
reference implementation** — read `module/capability/contract/csv/`
before starting; your module is that module with the format changed.

### The base you implement

`Contract` (`module/capability/contract/.src/lib.rs`):

```rust
pub trait Contract: Send + Sync {
    fn descriptor(&self) -> &ContractDescriptor;                       // id, version, representation
    fn identify(&self, stream: &Stream) -> Result<bool, ContractError>;   // is this my format?
    fn validate(&self, stream: &Stream) -> Result<ValidationResult, ContractError>;
}
```

`identify` answers "does this stream look like mine" (cheaply); `validate`
answers "and is it well-formed", returning `ValidationIssue`s rather than a bare
false so an operator sees *what* failed.

### Create, mount, land

Exactly as a transport (`module/capability/transport/doc/adding-a-transport.md`),
substituting `contract` for `transport`:

- repo `xmip-core-contract-<name>`, mounted at `<name>` directly inside the
  contract capability's repository;
- `Cargo.toml` depends on `contract = { package = "xmip-core-contract", … }`
  (and `stream`, for the `Stream` type);
- declare `[xmip.core.contract.<name>]` in `architecture.toml`.

How the repository is created, mounted and landed is the estate's:
`doc/architecture/repository-model.md` sections 7, 8 and 10.

The Playground pairs every contract against every transport, so a new contract is
picked up by pingpong the same way a new transport is.

---

