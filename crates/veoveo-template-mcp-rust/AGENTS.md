# Template MCP Server — Agent Manual

Delta over the repository root `AGENTS.md`. [`DESIGN.md`](DESIGN.md) is the design and
operational contract.

## Purpose

A tenant's named items: `create_item` records one, `template://item/{item_id}` reads it
and `template://items` lists them. The domain is the smallest that exercises a tool with
a product, an exact read and a keyset listing. A server created from the template
replaces it with its own.

## Invariants

- Owns the `template://` URI scheme and the PostgreSQL schema `template`, and its SQL
  touches no other schema. Identity: slug `template`, MCP `/template/mcp`, port 8810.
- A caller's tenant comes from the forwarded identity. A tool argument never names it,
  and every statement binds it, so another tenant's item reads as missing.
- Every target has one permission in `src/mcp/requirements.rs`. Handlers check that
  table, the registration declares it, and a test holds the tool list and the tool
  annotations to it.
- Addresses come from `TemplateResource` through the shared route declaration. Do not
  build or split a URI by hand.
- `create_item` returns its product through `product_result`, whose `resultUri` equals
  the resource link.
- Every statement is a `.sql` file under `src/store/items/` run through sqlx's checked
  macros. Regenerate `.sqlx` after changing a query or a migration.

## Build And Test

- `cargo test -p veoveo-template-mcp-rust` with the PostgreSQL 18 server programs on
  `PATH`, as the `dev` image provides them.
- `cargo test -p veoveo-template-mcp-rust --no-default-features --features contract`
  checks the contract alone.
- `tests/mcp.rs` calls the server through the SDK's test Gateway against a migrated
  database.
- `tests/database_conformance.rs` runs core's database checks over every statement and
  the `template` schema.
- `docker build --target check .` runs the gate.

## Contract Compliance

<!-- veoveo:contract-compliance:start -->
Contract: https://veoveo.ai/contracts/hosted-mcp

- C01: met
- C02: met
- C03: met — the template runs no durable operation; a server that does runs it as a task-augmented tool through the SDK's TaskWorker
- C04: met — items page by key at 100 per page, filtered by tenant inside each statement
- C05: met
- C06: met
- C07: met
- C08: met
- C09: met
- C10: met
- C11: met — the template calls no other server
- C12: met
- C13: met
- C14: met
- C15: pending — the OCI image builds; the Helm chart is pending
- C16: met
- C17: met
- C18: met
- C19: met
- C20: met
- C21: met
- C22: met
- C23: met
- C24: pending — a server created from the template takes its repository name, veoveo-mcp-<name>
- C25: met
- C26: met
- C27: met
- C28: met
- C29: met
- C30: met
- C31: pending — installed Discover and list readiness qualification is pending
- C33: pending — Owner naming producers, consumers and installed qualification are pending.
<!-- veoveo:contract-compliance:end -->
