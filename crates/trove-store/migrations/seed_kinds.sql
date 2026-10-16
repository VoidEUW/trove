-- The kinds every installation ships with, at fixed ids (ADR 6, ADR 9). `trove_core::seed_kinds()`
-- lists the same rows and a test keeps the two in step. A replay puts these rows back.
INSERT INTO kinds (id, label_key, counting, container) VALUES
    ('00000000-0000-7000-8000-000000000001', 'kind.electrical', 'single', 0),
    ('00000000-0000-7000-8000-000000000002', 'kind.furniture', 'single', 0),
    ('00000000-0000-7000-8000-000000000003', 'kind.artwork', 'single', 0),
    ('00000000-0000-7000-8000-000000000004', 'kind.documents', 'counted', 0),
    ('00000000-0000-7000-8000-000000000005', 'kind.memories', 'counted', 0),
    ('00000000-0000-7000-8000-000000000006', 'kind.valuables', 'single', 0),
    ('00000000-0000-7000-8000-000000000007', 'kind.software_key', 'single', 0),
    ('00000000-0000-7000-8000-000000000008', 'kind.parts_and_manuals', 'counted', 0),
    ('00000000-0000-7000-8000-000000000009', 'kind.tools', 'single', 0),
    ('00000000-0000-7000-8000-00000000000a', 'kind.other', 'counted', 0),
    ('00000000-0000-7000-8000-00000000000b', 'kind.box', 'single', 1);
