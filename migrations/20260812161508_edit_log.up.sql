-- Generic, append-only audit trail for the catalog-editing capability.
-- entity_type/entity_id are deliberately not a typed FK: one polymorphic
-- log beats four near-identical per-entity log tables (design.md's
-- Editing & deletion decision). old_value/new_value are full-row JSON
-- snapshots; new_value is null for a delete (nothing to move "to").
-- This table is the only recovery path in v1 — read via direct SQL by an
-- admin, no in-app revert/restore UI (content-moderation capability).
CREATE TABLE edit_log (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('company', 'store', 'product', 'store_product', 'image')),
    entity_id   BIGINT NOT NULL,
    action      TEXT NOT NULL CHECK (action IN ('update', 'delete')),
    old_value   JSONB NOT NULL,
    new_value   JSONB,
    changed_by  BIGINT REFERENCES "user" (id) ON DELETE SET NULL,
    changed     TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Looking up a given row's history is the primary access pattern for a
-- future admin revert/restore tool.
CREATE INDEX edit_log_entity_idx ON edit_log (entity_type, entity_id);
CREATE INDEX edit_log_changed_by_idx ON edit_log (changed_by);
