-- Drop the `category` taxonomy. It sat between the visitor and the thing
-- they actually came for: a first `<select>` to work through before the
-- product one offered anything narrower, a required field on every product
-- submission, and a "Kategorie" cell on each product card that mostly
-- restated what the product's own name and emoji already said.
--
-- Products keep their name, description and icon — everything the category
-- was ever shown next to — and search now filters by product directly.

ALTER TABLE product DROP COLUMN category;

DROP TABLE category;
