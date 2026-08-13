-- Per-listing seasonal availability — a JSONB array of month numbers
-- (1 = January .. 12 = December) the product is available in. NULL
-- means "available all year" (the default/common case — most listings
-- shouldn't need to touch this at all, see the new-listing form's
-- "only available seasonally" shortcut), not an empty array: an empty
-- array would mean "available in zero months", a different and useless
-- statement nothing should ever actually store.
ALTER TABLE store_product ADD COLUMN seasonal_months JSONB;
