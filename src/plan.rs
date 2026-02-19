//! src/plan.rs
//! Official Zorbs Development Plan & Roadmap
//! Update checkboxes as we complete items (use [x] or [✓])

pub const MVP_REMAINING: &str = r#"
# Zorbs MVP (Shippable Registry)

## Remaining Items (6 total)
- [ ] 1. Download endpoint  
  `GET /@name/version/download` — serves real tarball with proper headers & caching
- [ ] 2. Real zorb.toml parsing on publish  
  Auto-extract metadata, features, dependencies from uploaded tarball
- [ ] 3. Bootstrap zorb CLI (Rust reference impl)  
  `zorb new`, `zorb add`, `zorb publish`, `zorb install`
- [ ] 4. Dockerfile + docker-compose.yml  
  One-command dev & production setup
- [ ] 5. zorb.lock generation  
  CLI + registry produce reproducible lockfiles
- [ ] 6. Basic security & validation  
  File size limits, path sanitization, zorb.toml validation

## Completed
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
Current status: 7/13 MVP items complete (54%)
Next priority: Download endpoint + real parsing
"#;
