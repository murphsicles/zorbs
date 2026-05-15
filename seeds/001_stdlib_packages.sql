-- Seed: Official Zeta Standard Library packages
-- Populates a fresh registry database with the baseline 6 official packages.
--
-- Run AFTER migrations:
--   make seed
-- or:
--   psql "$DATABASE_URL" -f seeds/001_stdlib_packages.sql

INSERT INTO zorbs (id, name, version, description, license, repository, dependencies, downloads, created_at, updated_at)
VALUES
  ('bb7494ad-74bb-4755-a364-0350d7b29995', '@async/tokio',    '0.3.10', 'Epoll-based multi-threaded async runtime for Zeta with reactor, waker, timerfd', 'MIT',               'https://github.com/murphsicles/tokio',     '{}',                                                        0, '2026-05-07 07:52:14.506593+00', '2026-05-11 00:22:55.864894+00'),
  ('dda6b348-c7d0-417c-b18d-52db4be8c026', '@cli/clap',       '4.5.0',  'Command line argument parser',                                                    'MIT OR Apache-2.0', 'https://github.com/zeta-lang/clap',        '{}',                                                        0, '2026-05-07 07:52:14.510962+00', '2026-05-07 07:52:14.510962+00'),
  ('a7f27983-8cfc-42aa-ba6f-b5402e2a578e', '@core/once_cell', '1.0.0',  'Single assignment cells and lazy values for Zeta.',                               'MIT OR Apache-2.0', 'https://github.com/murphsicles/once_cell', '{}',                                                        0, '2026-05-14 23:55:41.616911+00', '2026-05-15 00:14:22.112998+00'),
  ('6ea6d0c3-7071-4ed1-acb8-702305772b7d', '@data/serde',     '0.4.0',  'Serialization/Deserialization framework for Zeta',                                'MIT',               'https://github.com/murphsicles/serde',     '{}',                                                        0, '2026-05-07 07:52:14.493674+00', '2026-05-07 07:52:14.493674+00'),
  ('be71b438-3b05-4fb5-a369-e29427dea349', '@http/axum',      '0.8.1',  'Ergonomic web framework',                                                         'MIT',               'https://github.com/zeta-lang/axum',        '{"@http/hyper": "^1.3", "@async/tokio": "^1.42"}',        0, '2026-05-07 07:52:14.508126+00', '2026-05-07 07:52:14.508126+00'),
  ('2c53bfa0-c204-4938-b3d2-6e2d9236d943', '@log/tracing',    '0.2.5',  'Structured, performant logging',                                                 'MIT',               'https://github.com/zeta-lang/tracing',     '{"@core/once_cell": "^1.19"}',                             0, '2026-05-07 07:52:14.510041+00', '2026-05-07 07:52:14.510041+00')
ON CONFLICT (name, version) DO NOTHING;
