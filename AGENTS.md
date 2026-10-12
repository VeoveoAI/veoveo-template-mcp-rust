# Agent Instructions

These rules apply to this repository and to component repositories created from it.

- Use upstream runtimes such as OpenShell and SurrealDB unmodified. Adapt through
  supported configuration.
- Prefer strong types: typed structs, enums and domain identifiers over raw strings and
  JSON wherever the shape is known.
- Keep test logic in Rust and tool behavior in each tool's own configuration. Do not add
  a task runner or shell scripts that hold test logic.
- Pin exact versions and image digests. When changing a pin, verify the latest stable
  release at its source.
- Run `docker build --target check .` on macOS and on Linux before pushing.
- Names, identifiers and formats follow the
  [naming and format conventions](https://github.com/VeoveoAI/veoveo-core/blob/main/docs/CONVENTIONS.md).
  Where a standard governs a name or format, follow it; extend a standard only through
  its own extension mechanism. URIs, paths and identifiers carry no version segment such
  as `/v1`; releases carry versions.
- Commit small, coherent changes. Commits carry no AI or tool attribution.
