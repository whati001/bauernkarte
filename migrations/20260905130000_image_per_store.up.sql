-- Re-home photos from the offering to the shop. A photo of a farm shop
-- is a photo of the *place* — the storefront, the shelf, the stand by
-- the road — not of one product listing, which is why the detail page
-- already flattened every product's photos into a single store-wide
-- carousel and used the first of them as the store's header image.
-- Hanging them off `store_product` meant the uploader had to pick a
-- product first, and a shop's photo count was split across however many
-- listings happened to carry one.

ALTER TABLE image ADD COLUMN store BIGINT REFERENCES store (id);

UPDATE image i SET store = sp.store
FROM store_product sp WHERE sp.id = i.store_product;

ALTER TABLE image ALTER COLUMN store SET NOT NULL;

-- Drops `image_store_product_idx` along with the column.
ALTER TABLE image DROP COLUMN store_product;

CREATE INDEX image_store_idx ON image (store);
