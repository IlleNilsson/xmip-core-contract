# Adding a contract

Moved here from the estate root on 2026-09-12 (ADR-0020 clause 3: the document lives where its subject lives).


Identical shape to a transport, against a different base. **`csv` is the
reference implementation** — read `module/core/capability/contract/csv/`
before starting; your module is that module with the format changed.

### The base you implement

`Contract` (`module/core/capability/contract/.src/lib.rs`):

```rust
pub trait Contract: Send + Sync {
    fn descriptor(&self) -> &ContractDescriptor;                       // id, version, representation
    fn identify(&self, stream: &Stream) -> Result<bool, ContractError>;   // is this my format?
    fn validate(&self, stream: &Stream) -> Result<ValidationResult, ContractError>;
}
```

`identify` answers "does this stream look like mine" (cheaply); `validate`
answers "and is it well-formed", returning `ValidationIssue`s rather than a bare
false so an operator sees *what* failed. Both read text through
`Stream::text`, decoded once for the two of them; an issue's code is a
`&'static str` where the technology names it, and its place is a
`contract::place::Place` walked down the content and spelled only when an
issue is raised. Whatever the reference names — a schema, a message type — is
read and compiled in `load`, once; `validate` only walks the Stream.

`ContractFactory` makes the contract a Location names, and declares what a
Location gives it (`fn settings`, ADR-0064 amendment 2026-09-26): the same
`xcore::settings::Settings` a transport declares — its `reference` where it
takes one (a schema, a message type, a copybook), anything else it reads,
each with its kind, default or requirement and meaning, and `technology:
env!("CARGO_PKG_NAME")`. `open` reads a Location's `contract_settings`
through it and hands the result to `configured`, which is `load(reference)`
unless the technology reads more; `load` keeps the one string the C ABI
carries (`export.rs`). The declaration is the contract's form in VS Code and
the desktop editor and what `configure` holds a Location to at start, so a
setting is never parsed by hand and a default never written twice.

### Create, mount, land

Exactly as a transport (`module/core/capability/transport/doc/adding-a-transport.md`),
substituting `contract` for `transport`:

- repo `xmip-core-contract-<name>`, mounted at `<name>` directly inside the
  contract capability's repository;
- `Cargo.toml` depends on `contract = { package = "xmip-core-contract", … }`
  (and `stream`, for the `Stream` type);
- declare `[xmip.core.contract.<name>]` in `architecture.toml`.

How the repository is created, mounted and landed is the estate's:
`doc/architecture/repository-model.md` sections 7, 8 and 10.

The Playground pairs every contract against every transport, so a new contract is
picked up by RoundTrip the same way a new transport is.

### Reaching a node

Not yet. A node refuses to start a Location that names a contract
([decided, not built](../../../../../doc/architecture/estate-map.md#arrival-validation)), so a contract is proven in
the Playground's harness and through the runtime's Module loader, which
configures, starts and stops a loaded contract table and is not in
`xmip-service` ([built, not in the assembled service](../../../../../doc/architecture/estate-map.md#module-loading)).

---

