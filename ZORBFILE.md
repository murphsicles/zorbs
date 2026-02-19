# Zorbfile Format (zorb.toml)

**The official manifest format for every Zeta zorb.**

One file. Zero boilerplate. Everything the CLI, registry, and ecosystem needs.

## Complete Example

```toml
[package]
name = "@zeta/axum"
version = "0.8.1"
edition = "2026"
authors = ["The Zeta Foundation <team@z-lang.org>"]
description = "Ergonomic, high-performance HTTP server for Zeta"
license = "MIT OR Apache-2.0"
repository = "https://github.com/zeta-lang/axum"
keywords = ["http", "web", "server", "async"]
categories = ["web-programming::http-server"]

[dependencies]
@zeta/tokio = "1.42"
@http/hyper = { version = "1.3", default-features = false }
@data/serde = { version = "1.0", features = ["derive"] }
@logging/tracing = { version = "0.2", optional = true }

[dev-dependencies]
@testing/reqwest = "0.12"

[features]
default = ["tracing"]
tracing = ["dep:tracing"]
full = ["tracing", "serde"]

[fmt]
style = "zeta-strict"
max_width = 100
use_tabs = false

[lint]
tool = "zippy"
level = "pedantic"
deny = ["unsafe_code"]
warn = ["style", "perf"]
allow = ["clippy::needless_range_loop"]

[build]
script = "build.zeta"

[workspace]
members = ["examples/*", "benches"]
```

## Specification

### [package] (required)

| Field          | Type          | Required | Description |
|----------------|---------------|----------|-------------|
| `name`         | string        | Yes      | Scoped name: `@org/name` or `name` |
| `version`      | semver string | Yes      | Semantic version |
| `edition`      | string        | Yes      | `2026` (future-proof) |
| `authors`      | array/string  | No       | Authors |
| `description`  | string        | No       | Short description |
| `license`      | string        | No       | SPDX identifier |
| `repository`   | string        | No       | Git URL |
| `keywords`     | array         | No       | Search keywords |
| `categories`   | array         | No       | zorbs.io categories |

### [dependencies] & [dev-dependencies]

- Key = scoped name (`@scope/name`)
- Value = version string **or** table with `version`, `features`, `default-features`, `optional`, `git`, `branch`, `rev`, `path`

### [features]

Standard feature flags:
- `default = ["tracing"]`
- Feature name = array of other features or `dep:xxx` for optional deps

### [fmt] section

```toml
style = "zeta-strict" | "zeta-pretty" | "custom"
max_width = 100
use_tabs = false
indent_size = 4
```

### [lint] section (Zippy)

```toml
tool = "zippy"
level = "recommended" | "pedantic" | "strict"
deny = ["unsafe_code", "todo"]
warn = ["style", "perf"]
allow = ["clippy::needless_range_loop"]
```

### [build] section

```toml
script = "build.zeta"
```

### [workspace] section

```toml
members = ["crates/*", "examples/*"]
```

## Rules enforced by `zorb publish`

- `name` must be unique globally (or under your org)
- Version must follow semver
- `zorb.toml` must be valid TOML + pass Zippy lint + fmt check
- `zorb.lock` is auto-generated and committed

---

**This is now the canonical Zorbfile specification.**

