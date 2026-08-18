-- Repairs a catalog whose products all render as the neutral package.
--
-- `product_icon` set icons with a one-shot UPDATE. The products it was
-- written for arrive from `scripts/seed_osm_farm_shops.py`, which runs
-- *after* migrations — so on any database built the documented way
-- (`bootstrap.py db` then `bootstrap.py stores`) that UPDATE matched
-- nothing and every product ended up with `icon` NULL.
--
-- Same mapping as `product_icon`, plus the two names the seed can emit
-- that it missed (Erdbeeren, Kirschen). `WHERE icon IS NULL` so an icon
-- set by hand since then is left alone. The seed script now inserts the
-- icon itself and carries the same table, which is what stops a freshly
-- built database needing this again.
UPDATE product SET icon = CASE name
    WHEN 'Äpfel'           THEN '🍎'
    WHEN 'Obst'            THEN '🍇'
    WHEN 'Gemüse'          THEN '🥦'
    WHEN 'Kartoffeln'      THEN '🥔'
    WHEN 'Erdbeeren'       THEN '🍓'
    WHEN 'Kirschen'        THEN '🍒'
    WHEN 'Kräuter'         THEN '🌿'
    WHEN 'Kürbiskerne'     THEN '🎃'
    WHEN 'Kürbiskernöl'    THEN '🛢️'
    WHEN 'Eier'            THEN '🥚'
    WHEN 'Käse'            THEN '🧀'
    WHEN 'Milch'           THEN '🥛'
    WHEN 'Milchprodukte'   THEN '🥛'
    WHEN 'Joghurt'         THEN '🥣'
    WHEN 'Butter'          THEN '🧈'
    WHEN 'Buttermilch'     THEN '🥛'
    WHEN 'Topfen'          THEN '🥣'
    WHEN 'Fleisch'         THEN '🥩'
    WHEN 'Rindfleisch'     THEN '🐄'
    WHEN 'Schweinefleisch' THEN '🐷'
    WHEN 'Speck'           THEN '🥓'
    WHEN 'Wurst'           THEN '🌭'
    WHEN 'Schinken'        THEN '🍖'
    WHEN 'Hühnerfleisch'   THEN '🍗'
    WHEN 'Brot'            THEN '🍞'
    WHEN 'Backwaren'       THEN '🥐'
    WHEN 'Honig'           THEN '🍯'
    WHEN 'Wein'            THEN '🍷'
    WHEN 'Saft'            THEN '🧃'
    WHEN 'Sirup'           THEN '🧴'
    WHEN 'Tee'             THEN '🍵'
    WHEN 'Likör'           THEN '🥃'
    WHEN 'Nudeln'          THEN '🍝'
    WHEN 'Getreide'        THEN '🌾'
    WHEN 'Gewürze'         THEN '🧂'
    WHEN 'Algen'           THEN '🌊'
    WHEN 'Blumen'          THEN '💐'
    WHEN 'Christbaum'      THEN '🎄'
    WHEN 'Suppen'          THEN '🍲'
    WHEN 'Marmelade'       THEN '🫙'
    WHEN 'Fisch'           THEN '🐟'
    ELSE NULL
END
WHERE icon IS NULL;
