-- Reverting needs the duplicates gone first, or the unconditional index
-- can't be built; nothing to do automatically here beyond the swap, so a
-- failure means there are live+deleted rows sharing a name.
DROP INDEX product_name_key;
CREATE UNIQUE INDEX product_name_key ON product (name);
