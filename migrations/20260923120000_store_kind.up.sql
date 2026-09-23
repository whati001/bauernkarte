-- What kind of place a store is — it decides the map pin. `market` is a
-- (farmers') market, `vending_machine` a self-service machine, `shop` a
-- farm shop. Existing rows default to `market`: nothing recorded says
-- otherwise, and re-running the OSM seed classifies the rows it created.
ALTER TABLE store
    ADD COLUMN kind TEXT NOT NULL DEFAULT 'market'
        CHECK (kind IN ('market', 'vending_machine', 'shop'));
