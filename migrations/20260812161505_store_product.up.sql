CREATE TABLE store_product (
    id           BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    store        BIGINT NOT NULL REFERENCES store (id),
    product      BIGINT NOT NULL REFERENCES product (id),
    price        NUMERIC(10, 2) NOT NULL CHECK (price >= 0),
    approved     BOOLEAN NOT NULL DEFAULT false,
    deleted      BOOLEAN NOT NULL DEFAULT false,
    created_by   BIGINT REFERENCES "user" (id) ON DELETE SET NULL,
    modified_by  BIGINT REFERENCES "user" (id) ON DELETE SET NULL,
    created      TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX store_product_store_idx ON store_product (store);
CREATE INDEX store_product_product_idx ON store_product (product);
CREATE INDEX store_product_created_by_idx ON store_product (created_by);
CREATE INDEX store_product_modified_by_idx ON store_product (modified_by);
