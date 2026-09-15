-- Cannot invert exactly: a store-level photo names no single listing, so
-- each is re-pointed at its store's lowest-numbered offering. Photos on a
-- store with no offerings at all have nowhere to go under the old NOT
-- NULL FK and are dropped.
ALTER TABLE image ADD COLUMN store_product BIGINT REFERENCES store_product (id);

UPDATE image i SET store_product = (
    SELECT sp.id FROM store_product sp WHERE sp.store = i.store ORDER BY sp.id LIMIT 1
);

DELETE FROM image WHERE store_product IS NULL;

ALTER TABLE image ALTER COLUMN store_product SET NOT NULL;
ALTER TABLE image DROP COLUMN store;

CREATE INDEX image_store_product_idx ON image (store_product);
