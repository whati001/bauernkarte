-- Needed so seed/import scripts (and, incidentally, the "add existing vs.
-- new product" form) can safely upsert a shared catalog entry by name
-- instead of accumulating duplicate "Äpfel" rows across many stores.
CREATE UNIQUE INDEX product_name_key ON product (name);
