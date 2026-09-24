-- The uploader's "use as store image": the header photo on the store
-- panel and the thumbnail in the search list. Several images may carry
-- it (each upload can claim it, and needs approval first); the newest
-- approved one wins, and a store without one falls back to its oldest
-- photo, as before.
ALTER TABLE image ADD COLUMN cover BOOLEAN NOT NULL DEFAULT false;
