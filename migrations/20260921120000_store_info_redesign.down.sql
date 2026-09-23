DROP TABLE store_review;

ALTER TABLE image DROP COLUMN kind;

ALTER TABLE store
    DROP COLUMN address,
    DROP COLUMN phone,
    DROP COLUMN owner_name,
    DROP COLUMN owner_since,
    DROP COLUMN owner_bio;
