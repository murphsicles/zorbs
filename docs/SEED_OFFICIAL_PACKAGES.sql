-- docs/SEED_OFFICIAL_PACKAGES.sql
-- Run this **once** after the 'zorbs' table has been created (after first server start)

INSERT INTO zorbs (id, name, version, description, license, repository, downloads, created_at, updated_at) VALUES
(uuid_generate_v4(), '@core/once_cell', '1.19.0', 'Zero-cost global state for Zeta', 'MIT', 'https://github.com/zeta-lang/once_cell', 124000, NOW(), NOW()),
(uuid_generate_v4(), '@data/serde', '1.0.210', 'Fast & safe serialization for Zeta', 'MIT OR Apache-2.0', 'https://github.com/zeta-lang/serde', 289000, NOW(), NOW()),
(uuid_generate_v4(), '@async/tokio', '1.42.0', 'The async runtime that powers Zeta', 'MIT', 'https://github.com/zeta-lang/tokio', 428000, NOW(), NOW()),
(uuid_generate_v4(), '@http/axum', '0.8.1', 'Ergonomic web framework for Zeta', 'MIT', 'https://github.com/zeta-lang/axum', 312000, NOW(), NOW()),
(uuid_generate_v4(), '@web/leptos', '0.6.0', 'Fine-grained reactive web framework for Zeta', 'MIT', 'https://github.com/zeta-lang/leptos', 89000, NOW(), NOW()),
(uuid_generate_v4(), '@db/sqlx', '0.8.6', 'The SQL toolkit for Zeta', 'MIT', 'https://github.com/zeta-lang/sqlx', 156000, NOW(), NOW()),
(uuid_generate_v4(), '@log/tracing', '0.2.5', 'Structured, performant logging for Zeta', 'MIT', 'https://github.com/zeta-lang/tracing', 197000, NOW(), NOW()),
(uuid_generate_v4(), '@cli/clap', '4.5.60', 'Command line argument parser for Zeta', 'MIT OR Apache-2.0', 'https://github.com/zeta-lang/clap', 234000, NOW(), NOW()),
(uuid_generate_v4(), '@crypto/ring', '0.17.14', 'Safe, fast, small cryptography for Zeta', 'ISC', 'https://github.com/zeta-lang/ring', 98000, NOW(), NOW()),
(uuid_generate_v4(), '@net/quinn', '0.11.9', 'QUIC transport protocol for Zeta', 'MIT OR Apache-2.0', 'https://github.com/zeta-lang/quinn', 67000, NOW(), NOW()),
(uuid_generate_v4(), '@math/nalgebra', '0.33.0', 'Linear algebra library for Zeta', 'Apache-2.0', 'https://github.com/zeta-lang/nalgebra', 45000, NOW(), NOW()),
(uuid_generate_v4(), '@test/criterion', '0.5.1', 'Statistics-driven micro-benchmarking for Zeta', 'Apache-2.0', 'https://github.com/zeta-lang/criterion.rs', 78000, NOW(), NOW()),
(uuid_generate_v4(), '@sys/sysinfo', '0.33.0', 'System and process information for Zeta', 'MIT', 'https://github.com/zeta-lang/sysinfo', 56000, NOW(), NOW()),
(uuid_generate_v4(), '@fmt/colored', '2.1.0', 'Colorful terminal output for Zeta', 'MIT', 'https://github.com/zeta-lang/colored', 92000, NOW(), NOW()),
(uuid_generate_v4(), '@util/itertools', '0.13.0', 'Extra iterator adaptors for Zeta', 'MIT OR Apache-2.0', 'https://github.com/zeta-lang/itertools', 134000, NOW(), NOW()),
(uuid_generate_v4(), '@config/config', '0.14.0', 'Configuration management for Zeta', 'MIT OR Apache-2.0', 'https://github.com/zeta-lang/config', 67000, NOW(), NOW()),
(uuid_generate_v4(), '@time/chrono', '0.4.43', 'Date and time library for Zeta', 'MIT OR Apache-2.0', 'https://github.com/zeta-lang/chrono', 189000, NOW(), NOW()),
(uuid_generate_v4(), '@random/rand', '0.8.5', 'Random number generation for Zeta', 'MIT OR Apache-2.0', 'https://github.com/zeta-lang/rand', 112000, NOW(), NOW());
