-- Kinds and items (ADR 5, ADR 6). Both are projections of the operation log; the seed kinds that
-- every installation ships with are inserted by seed_kinds.sql, which follows this file.

CREATE TABLE kinds (
    id        TEXT    PRIMARY KEY,
    -- A seed kind stores a message key the UI translates (ADR 15); a kind the user named stores the
    -- name as typed. Exactly one of the two is set.
    label_key TEXT,
    name      TEXT,
    counting  TEXT    NOT NULL CHECK (counting IN ('single', 'counted')),
    container INTEGER NOT NULL DEFAULT 0 CHECK (container IN (0, 1)),
    archived  INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    CHECK ((label_key IS NULL) <> (name IS NULL))
) STRICT;

CREATE TABLE items (
    id         TEXT    PRIMARY KEY,
    name       TEXT    NOT NULL CHECK (length(trim(name)) > 0),
    kind_id    TEXT    NOT NULL REFERENCES kinds (id),
    quantity   INTEGER NOT NULL CHECK (quantity >= 1),
    -- Unix milliseconds of the first and the latest operation that touched the item.
    created_ms INTEGER NOT NULL,
    updated_ms INTEGER NOT NULL,
    archived   INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1))
) STRICT;

CREATE INDEX items_created ON items (created_ms);
