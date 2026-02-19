# Zorbs
**Zorbs is an awesome new package manager for the Zeta language.**
<div align="center">
![Zorbs Hero](/assets/hero.jpg)
**Build. Release. Share.**
</div>

![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Crates.io](https://img.shields.io/crates/v/zorbs.svg)
![Dependencies](https://deps.rs/repo/github/murphsicles/zorbs/status.svg)

## The Vision
Zeta is ready. The world needs a package manager that doesn't just work — it **inspires**.
Zorbs is the gateway for developers to **share, build, and release** libraries with groundbreaking awesomeness and dead-simple efficiency. No bloat. No drama. Just pure velocity and unbreakable trust.
We took everything Rust’s Cargo and crates.io got right… and fixed everything they didn’t.

## Why Zorbs Crushes the Competition
| Feature | crates.io (Rust) | Zorbs (Zeta) |
|--------------------------|---------------------------|---------------------------------------|
| **Security** | Manual vigilance | Automatic scanning + SLSA Level 3 + trusted publishing from day one |
| **Discovery** | Basic search | AI semantic search + visual dep graph + health badges |
| **Install Speed** | Recompile everything | Pre-built binaries + content-addressable cache (sub-2s installs) |
| **Namespaces** | Flat & squattable | `@org/name` or `user/name` with ownership transfers |
| **Docs** | Separate site | Built-in, versioned, searchable docs at `docs.zorbs.io` |
| **Reproducibility** | Good | Bit-for-bit, cross-machine, offline-first |
| **Publishing** | Token hassle | One-command from any CI with OIDC |

## Core Features
### 1. Security That Feels Invisible
- Every zorb auto-scanned on publish (malware, CVEs, static analysis)
- Trusted Publishing (GitHub, GitLab, any OIDC) — no secrets ever stored
- `zorb install` refuses known-bad versions unless you force it
- Immutable, signed, content-addressable storage

### 2. Discovery That Feels Like Magic
- “HTTP/2 server with TLS 1.3 for embedded Zeta” → instant results
- Interactive dependency graph explorer
- Live health scores, trending, “Zeta Weekly Picks”
- Automatic, beautiful, cross-linked documentation

### 3. Efficiency That Makes Disk Space Irrelevant
- Global deduplicated cache (hard links)
- Pre-built binaries for every target (x86_64, aarch64, riscv, wasm…)
- Parallel resumable downloads + perfect offline mode
- `zorb.lock` guarantees exact reproduction forever

### 4. Namespaces & Ownership Done Right
- Clean `@team/zorb-name` syntax
- Organizations, teams, fine-grained permissions
- Verified name reservations & instant transfers with full audit log

### 5. Publishing That Feels Like Breathing
```bash
zorb init my-awesome-lib
# → beautiful template with tests, CI, docs
zorb publish
# → runs on 8 platforms, scans, signs, ships
```

## Quick Start
```bash
# Install the CLI (one-liner, works on Linux/macOS/Windows)
curl -fsSL https://install.zorbs.io | sh
# Create your first project
zorb init my-project
cd my-project
# Add a zorb
zorb add @http/axum
# Build & run
zorb build
zorb run
```

Visit [zorbs.io](https://zorbs.io) to browse 10,000+ zorbs by 2027.

## Architecture (Simple & Bulletproof)
- **CLI**: Written in Zeta (Rust reference impl for bootstrap)
- **Registry**: Open-source, S3+CDN backend, git-style index for auditability
- **Storage**: Content-addressable + global CDN
- **Frontend**: Blazing-fast, mobile-first, dark-mode native

Private registries in **one command**:
```zeta
zorb registry new my-company
```

## Roadmap to World Domination
**MVP (Now)**
- Core CLI + basic registry + `zorb.toml` + reproducible builds

**v1 (Next 60 days)**
- Namespaces, prebuilts, security scanner, docs hosting, trusted publishing

**v2 (“Holy Shit”)**
- AI semantic search, visual graph explorer, private registry marketplace, zero-install monorepos

**Beyond**
- Ecosystem health tools, auto-fork suggestions, Zeta-native plugins, mobile app

## Contributing
We’re building the best package manager in the world — and we want **you**.
- Open issues
- Submit PRs
- Star the repo
- Publish your first zorb and tell the world
See [CONTRIBUTING.md](CONTRIBUTING.md) and join our Discord.

## License
MIT with ❤️ for the Zeta community.

---
**Zorbs is a new package manager for the Zeta language.**

Let’s make Zeta the most joyful systems language on the planet.

**Build. Release. Share.**
