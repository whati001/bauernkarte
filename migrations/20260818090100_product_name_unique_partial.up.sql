-- `product_name_key` had no predicate, so it also covered soft-deleted
-- rows: once "Äpfel" was deleted, the name was reserved forever and
-- re-creating it failed with a duplicate-key error. Restoring the
-- deleted row was safe, but the catalog had a dead end in it.
--
-- The predicate frees the name again. The cost is that a restore can now
-- collide (the name may have been taken meanwhile) — `db::moderation`
-- checks for that and reports it instead of surfacing a 500.
DROP INDEX product_name_key;
CREATE UNIQUE INDEX product_name_key ON product (name) WHERE NOT deleted;
