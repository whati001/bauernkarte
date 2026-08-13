-- Initial category taxonomy so local dev / early product submissions have
-- something to classify against. Admin-managed thereafter (direct SQL);
-- not exhaustive, just enough for a farm-shop/market domain to function.
INSERT INTO category (name) VALUES
    ('Obst & Gemüse'),
    ('Eier'),
    ('Milchprodukte & Käse'),
    ('Fleisch & Wurst'),
    ('Brot & Backwaren'),
    ('Honig'),
    ('Getränke'),
    ('Sonstiges')
ON CONFLICT DO NOTHING;
