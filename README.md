# veoveo-template-mcp-rust

This template holds the starting files for a Veoveo MCP server written in Rust. A new
component repository copies them once and owns them from then on. Components couple
only through contracts, conformance suites and the `rust-version` of the crates they
compile from other repositories, so each repository keeps its own image, toolchain
and gate configuration current.

## Contents

| Path | Purpose |
|---|---|
| `Dockerfile` | The `dev` environment, the `check` gate, the release `build` and the `runtime` image |
| `openshell/policy.yaml` | The OpenShell policy for the development environment |
| `env/profile.d/rust.sh` | Toolchain variables for login shells inside the environment |
| `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml` | The pinned toolchain and code rules |
| `deny.toml` | Dependency licenses, advisories, bans and allowed sources |
| `.config/nextest.toml` | Test profiles for local runs and machine-readable results |
| `crates/` | The component's crates; the template ships a placeholder |

## Start a component

Create the repository from this template, rename the placeholder crate to the
repository name and set `COMPONENT` in the `Dockerfile` to the binary name.

## Gate

```sh
docker build --target check .
```

The `check` stage runs rustfmt, clippy with warnings denied, cargo-deny, cargo-hack on
each feature and the nextest `ci` profile. It runs the same way on macOS and Linux.

## Development environment

Build the `dev` image on the host and create an OpenShell sandbox from it:

```sh
docker build --target dev -t <repository>:dev .
openshell sandbox create --name <sandbox> --from <repository>:dev \
  --policy openshell/policy.yaml --env HOME=/sandbox --detach -- sleep infinity
```

OpenShell limits sandbox names to 19 characters, so sandboxes and other runtime
identifiers use a short form: `vv-` plus a short kind and the component name. The
kinds are `srv` for services, `mcp` for MCP servers, `rt` for runtimes, `app`,
`sdk`, `tpl` for templates, `demo` for showcases and `qual` for qualification suites,
and repositories without a kind use `vv-<name>`. For example, `veoveo-service-gateway`
runs as `vv-srv-gateway` and `veoveo-mcp-time` as `vv-mcp-time`. Repositories,
crates, binaries and images keep their full names.

Inside the sandbox, the `sandbox` user owns the Rust toolchain and Cargo home, so
rustup and Cargo install components, targets and crates as needed. A coding agent on
the host works on the source through OpenShell SSH: it pushes commits into a working
tree at `/sandbox/src` that accepts pushes with `receive.denyCurrentBranch=updateInstead`,
then builds and tests inside the sandbox.

## License

Apache-2.0. See [LICENSE](LICENSE).
