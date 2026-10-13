-- The operation log (ADR 9, ADR 16) and the installation's key/value settings.
-- No entity tables: each entity adds its projection in its own migration.

CREATE TABLE ops (
    op_id      TEXT    PRIMARY KEY,
    device_id  TEXT    NOT NULL,
    hlc        INTEGER NOT NULL CHECK (hlc >= 0),
    op_version INTEGER NOT NULL CHECK (op_version >= 1),
    entity     TEXT    NOT NULL,
    entity_id  TEXT    NOT NULL,
    patch      TEXT    NOT NULL CHECK (json_valid(patch))
) STRICT;

-- Replay applies operations in (hlc, device_id) order.
CREATE INDEX ops_order ON ops (hlc, device_id);

CREATE TABLE meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT;
