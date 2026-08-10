# Architecture

> **This file is a contract, not documentation.** The `architecture-reviewer` agent reads it
> on every review and checks your diffs against the rules below. Vague rules produce vague
> reviews; specific rules catch real mistakes.
>
> Fill in each section and delete this block. If you are not sure yet, write down what the
> code *currently* does — an accurate description of a messy design is far more useful to the
> reviewer than an aspirational description of a clean one. Update it as the design changes;
> the `docs-updater` agent will flag it when it goes stale.

## What this project is

<!-- Two or three sentences. What it does, who uses it, what it is not trying to be. -->

_Example: A CLI that syncs local Markdown notes to a remote server. Single-user, offline-first.
Not a collaborative editor and not a web application._

## Pattern

<!-- Name the pattern you are following: MVC, hexagonal/ports-and-adapters, layered,
     pipeline, plain modules. If you are not following a named pattern, say so — that is a
     valid answer and the reviewer will judge placement by the dependency rules below. -->

**Pattern:** _e.g. Layered (domain / application / infrastructure)_

## Layers

<!-- One row per layer. "May depend on" is the rule the reviewer enforces most. Be exact —
     "may depend on: domain only" catches far more than "may depend on: lower layers". -->

| Layer            | Lives in           | Responsibility                                   | May depend on        |
| ---------------- | ------------------ | ------------------------------------------------ | -------------------- |
| _Domain_         | `src/domain/`      | _Core types and business rules. No I/O._         | _nothing_            |
| _Application_    | `src/app/`         | _Use cases; orchestrates domain and ports._      | _domain_             |
| _Infrastructure_ | `src/infra/`       | _Database, HTTP clients, filesystem, CLI._       | _domain, application_ |
| _Entry points_   | `src/main.rs`      | _Wiring and startup only. No business logic._    | _all_                |

## Rules

<!-- Absolute rules. The reviewer treats a violation of anything here as a BLOCKER, so only
     put things here you actually mean. Delete the ones that do not apply. -->

1. **Dependencies point inward.** Nothing in `domain/` imports from `app/` or `infra/`.
2. **No cycles between modules.** Ever.
3. **Business logic never lives in an entry point** — not in `main`, a route handler, a CLI
   command body, or a template.
4. **Infrastructure types do not escape their layer.** Database rows, ORM objects, and
   framework request/response types stop at the boundary; convert to domain types there.
5. **One way to do each thing.** One HTTP client, one config loader, one error type per
   domain. A second one is a finding.
6. _<add your own — the project-specific ones catch the most>_

## Where new code goes

<!-- The reviewer uses this to judge placement. A short lookup table beats prose. -->

| If you are adding…                | Put it in…            |
| --------------------------------- | --------------------- |
| _A new business rule_             | `src/domain/`         |
| _A new user-facing operation_     | `src/app/`            |
| _A call to an external service_   | `src/infra/clients/`  |
| _A new CLI subcommand_            | `src/cli/` (thin — delegates to `app/`) |

## Known deviations

<!-- Places where the code knowingly breaks the rules above, and why. Listing them here stops
     the reviewer from reporting the same known issue on every single pull request. Remove an
     entry when you fix it. -->

- _e.g. `src/infra/legacy_import.rs` reaches directly into the domain's internals. Predates the
  current layering; scheduled for rework, do not extend it._

## Decisions worth knowing

<!-- Short entries. Why a non-obvious choice was made, so neither you nor the agents relitigate
     it in six months. -->

- **_Decision:_** _e.g. Store timestamps as UTC integers, not ISO strings._
  **_Why:_** _e.g. Sorting and arithmetic in SQLite; formatting is a display concern._
