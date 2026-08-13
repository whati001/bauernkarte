-- Cosmetic, not structural: an emoji shown next to each product, same
-- treatment as the `category_icon` migration and for the same reason
-- (native <option> elements can't render markup, only text). Nullable:
-- a product added through the normal submission flow (or via direct SQL)
-- without a mapped name just gets the neutral package fallback the
-- templates apply.
ALTER TABLE product ADD COLUMN icon TEXT;

UPDATE product SET icon = CASE name
    WHEN 'Äpfel'           THEN '🍎'
    WHEN 'Obst'            THEN '🍇'
    WHEN 'Gemüse'          THEN '🥦'
    WHEN 'Kartoffeln'      THEN '🥔'
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
END;
