-- Drop the `company` layer. It existed so several shops could share one
-- business record, but every shop in practice *is* its own business:
-- the extra entity meant a second thing to pick before a store could be
-- submitted, a second moderation queue, and a "Firma"/"Geschäft" pair of
-- detail cards that repeated the same name twice.
--
-- The store's own name/position/opening hours carry everything the
-- company row did that visitors ever saw; description and homepage are
-- dropped with it (no store-level equivalent existed, and re-homing them
-- would silently attribute one company's text to each of its shops).

ALTER TABLE store DROP COLUMN company;

DROP TABLE company;

-- `edit_log` is polymorphic, so its CHECK still named the gone table and
-- its company rows now point at nothing.
DELETE FROM edit_log WHERE entity_type = 'company';
ALTER TABLE edit_log DROP CONSTRAINT edit_log_entity_type_check;
ALTER TABLE edit_log ADD CONSTRAINT edit_log_entity_type_check
    CHECK (entity_type IN ('store', 'product', 'store_product', 'image'));
