-- Repeated relations must retain each statement's own relation, including multiline SQL.
SELECT id FROM items;
SELECT
  items.id
FROM items;
SELECT id FROM "items";
EXPLAIN ANALYZE
SELECT id FROM items;
WITH first_read AS (
  SELECT id FROM items
)
SELECT id FROM items;
PREPARE read_items AS
SELECT id FROM items;
