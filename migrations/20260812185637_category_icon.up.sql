-- Cosmetic, not structural: an emoji shown next to each category in the
-- search filter's <select>. Native <option> elements can't render SVG (or
-- any markup) — only text — so this stores a plain emoji character rather
-- than referencing one of the vendored Lucide icons used everywhere else
-- in the UI. Nullable: an admin adding a category via direct SQL without
-- setting one just gets the neutral fallback the template applies.
ALTER TABLE category ADD COLUMN icon TEXT;

UPDATE category SET icon = CASE name
    WHEN 'Obst & Gemüse'          THEN '🥕'
    WHEN 'Eier'                   THEN '🥚'
    WHEN 'Milchprodukte & Käse'   THEN '🧀'
    WHEN 'Fleisch & Wurst'        THEN '🥩'
    WHEN 'Brot & Backwaren'       THEN '🍞'
    WHEN 'Honig'                  THEN '🍯'
    WHEN 'Getränke'               THEN '🥤'
    WHEN 'Sonstiges'              THEN '🧺'
    ELSE NULL
END;
