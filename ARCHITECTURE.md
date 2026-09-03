# Architecture

One repo, many views. The rule that carries the whole design:

> **Arrows point inward. Nothing in `crates/` may name a view.**

```
coer/
├── crates/
│   ├── model/          coer-model       types + ports (traits). Zero I/O.
│   ├── controller/     coer-controller  use cases, generic over the ports
│   ├── storage/        coer-storage     JsonStorage, later SqliteStorage
│   └── esv/            coer-esv         the ESV HTTP client
└── views/
    ├── cli/            coer-cli   (binary: coer)
    └── web/            coer-web
```

| crate | may depend on |
|---|---|
| `coer-model` | serde, thiserror. **Nothing else in this repo.** |
| `coer-controller` | `coer-model` |
| `coer-storage` | `coer-model` |
| `coer-esv` | `coer-model` |
| views | all four |

These are crates, not modules, because Cargo's compilation and dependency unit is
the crate. That makes the layering a compile error rather than a convention:
`coer-controller`'s manifest does not list `reqwest`, so `use reqwest::Client`
in there is `E0432`. Measured on this workspace:

| crate | packages in its build |
|---|---|
| `coer-model` | 13 |
| `coer-controller` | 15 |
| `coer-storage` | 22 |
| `coer-cli` | 216 |

That gap is why `cargo check -p coer-model` is instant and why the controller's
tests run with no network, no disk and no API key.

## Ports live with the model

`Storage` and `VerseSource` are defined in `coer-model/src/ports.rs`, not beside
the code that implements them. `coer-storage` depends on `coer-model` to learn
what it must provide; `coer-controller` depends on `coer-model` to say what it
needs. Neither ever names the other. The adapter depends on the abstraction, and
the abstraction belongs to the core.

## How a backend gets chosen

The controller is generic over its ports, so each view names the backends it
wants and the compiler builds a specialised copy — no vtable, no runtime cost:

```rust
// views/cli/src/main.rs
let coer = Coer::new(JsonStorage::new(data_dir()), EsvClient::from_env()?);

// views/web/src/main.rs — same controller, same call sites
let coer = Coer::new(SqliteStorage::connect(&url).await?, EsvClient::from_env()?);
```

Reach for `Box<dyn Storage>` only if the backend must be picked at *runtime*
(a `--storage=sqlite` flag). It is not needed for per-view choices.

## Guidelines

1. **Check the table before adding a dependency.** If something in `crates/`
   needs to know about a view, invert it into a trait instead.
2. **A new backend is a new `impl`, not a new method.** Adding Postgres should be
   one new file in `coer-storage` and one changed line in a view. If it forces a
   new method onto `Storage`, the trait was shaped around JSON rather than around
   the domain.
3. **The model reads no env vars, opens no files, prints nothing.** This is why
   there is no shared `config` module: `data_dir()` lives in `coer-storage`
   because it is a fact about filesystems, and `ESV_API_KEY` lives in `coer-esv`.
   Shared config modules are where layering quietly rots.
4. **Views own all wiring and setup.** `.env`, terminal init, the async runtime,
   choosing backends. A view is the only place two concrete adapters are named
   together.
5. **Wire types stay private to their adapter.** `EsvResponse` is `pub(crate)`
   in `coer-esv`, and their plain-text layout is parsed in `parse.rs`. When ESV
   changes their format, the blast radius is that crate.
6. **Nothing outside a view formats for humans.** `impl Display` on a domain type
   is a view concern.
7. **Each port owns its error enum; adapters map into it.** A missing file is
   `StorageError::NotFound`, not a leaked `io::Error` — a SQL backend has no
   such thing. Use `#[source]` so callers can unwind the chain.
8. **Keep `coer-model` small.** Everything depends on it, so a change there
   recompiles everything. Types and ports, nothing more.

## Commands

```sh
cargo run                       # the CLI (default-members)
cargo run -p coer-web           # the web view
cargo test --workspace          # everything — see the CI note below
cargo check -p coer-model       # the fast inner loop
```

`--workspace` is not optional in CI: the root is a virtual manifest with
`default-members = ["views/cli"]`, so a bare `cargo build` covers only the CLI.
