-- Price was never load-bearing for what this app is actually for (finding
-- which store carries which product) — dropped per explicit request
-- rather than kept as unused dead weight in the form/detail view.
ALTER TABLE store_product DROP COLUMN price;
