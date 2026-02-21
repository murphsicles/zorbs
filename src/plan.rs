//! src/plan.rs
//! Official Zorbs Development Plan & Roadmap
//! Update checkboxes as we complete items (use [x] or [✓])

pub const MVP_REMAINING: &str = r#"
# Zorbs MVP (Shippable Registry)
## Remaining Items (0 total)
## Completed
- [x] Basic security & validation
  File size limits (50 MB), path sanitization, zorb.toml validation, reserved scopes
- [x] zorb.lock generation
- [x] Bootstrap zorb CLI (Rust reference impl)
- [x] Download endpoint + tracking
- [x] Real zorb.toml parsing on publish
- [x] Dockerfile + docker-compose.yml
- [x] Modular Axum structure
- [x] Config, State, DB, Models, Handlers, Views, Routes
- [x] Homepage + live search
- [x] Publish page + form
- [x] Zorb detail page with version history
- [x] Dependencies display
- [x] Zorbfile specification
"#;

pub const POST_MVP_V1: &str = r#"
# Post-MVP v1 — "Holy Shit" Release (Next 60 days)

## Phase 1: Official Zeta Standard Library (Highest Priority)
- [ ] Reserve and seed all 18 Super Domains (@core, @data, @async, @http, etc.)
- [ ] Create first 6 official Zeta packages (@data/serde, @async/tokio, @http/axum, @core/once_cell, @log/tracing, @cli/clap)
- [ ] Add "Zeta Standard Library" section to homepage with official badges

## Phase 2: Trusted Publishing & Security
- [ ] Trusted Publishing (GitHub OIDC)
- [ ] Automatic security scanning + SLSA Level 3
- [ ] Organizations & private registries

## Phase 3: Core Engine
- [ ] Full dependency graph + resolution engine
- [ ] Pre-built binaries for all targets (x86_64, aarch64, wasm, etc.)
- [ ] AI semantic search

## Phase 4: Ecosystem Tools
- [ ] Automatic docs hosting (docs.zorbs.io)
- [ ] Ecosystem health tools (abandoned detection, badges, auto-forks)
"#;

pub const DARK_FACTORY: &str = r#"
# The Dark Factory — Autonomous Zeta Standard Library

## Phase 1: Foundation (Next 30 days)
- [ ] Create tools/dark-factory/ scaffolding
- [ ] Rust → Zeta transpiler skeleton
- [ ] Automated test generation + regression suite
- [ ] Benchmark harness (criterion-style)

## Phase 2: Automation (Next 60–90 days)
- [ ] Grok continuously scrapes top Rust crates
- [ ] Auto-transpile + idiomatic Zeta rewrite
- [ ] Full CI regression + benchmark comparison
- [ ] Auto-publish new versions to @domain/ scopes

## Phase 3: Self-Sustaining
- [ ] Dark Factory maintains all 18 super domains
- [ ] Version bumping, security patches, performance wins
- [ ] Public leaderboard of Zeta vs Rust performance
"#;

pub const STATUS: &str = r#"
Current status: MVP 100% shipped — Zorbs is live and production-ready.

Next priority: 
1. Official Zeta Standard Library (Super Domains)
2. Dark Factory foundation
3. Trusted Publishing

We are now entering the "Holy Shit" phase.
The crates are dead. Long live the zorbs.
"#;
