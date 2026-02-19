// src/plan.rs
//! src/plan.rs
//! Official Zorbs Development Plan & Roadmap
//! Update checkboxes as we complete items (use [x] or [✓])

pub const MVP_REMAINING: &str = r#"
# Zorbs MVP (Shippable Registry)
## Remaining Items (4 total)
- [ ] 1. Bootstrap zorb CLI (Rust reference impl)
  `zorb new`, `zorb add`, `zorb publish`, `zorb install`
- [ ] 2. zorb.lock generation
  CLI + registry produce reproducible lockfiles
- [ ] 3. Basic security & validation
  File size limits, path sanitization, zorb.toml validation
## Completed
- [x] Download endpoint
- [x] Real zorb.toml parsing on publish
- [x] Dockerfile + docker-compose.yml
  One-command dev & production setup
- [x] Modular Axum structure (main.rs ≤ 70 LOC)
- [x] Config, State, DB, Models, Handlers, Views, Routes
- [x] Homepage + live search
- [x] Publish page + form
- [x] Zorb detail page with version history
- [x] Dependencies display on detail page
- [x] Zorbfile specification (ZORBFILE.md)
"#;

pub const POST_MVP_V1: &str = r#"
# Post-MVP v1 ("Holy Shit" Release)
- [ ] Trusted Publishing (GitHub OIDC)
- [ ] Full dependency graph + resolution engine
- [ ] Automatic security scanning + SLSA Level 3
- [ ] Pre-built binaries for all targets
- [ ] Organizations & private registries
- [ ] AI semantic search
- [ ] Automatic docs hosting (docs.zorbs.io)
- [ ] Ecosystem health tools (abandoned detection, badges, auto-forks)
"#;

pub const STATUS: &str = r#"
Current status: 10/13 MVP items complete (77%)
Next priority: Bootstrap zorb CLI
"#;
