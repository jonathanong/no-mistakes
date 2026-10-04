-- These non-PostgreSQL partition spellings can survive generic AST recovery.
-- Neither may detach the valid child before its parent is dropped.
CREATE TABLE orders(id integer);
CREATE TEMP TABLE accounts(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE orders PARTITION OF accounts FOR VALUES FROM (0) TO (10);
ALTER TABLE accounts DETACH PART 1;
ALTER TABLE accounts DETACH PARTITION (1 + 1);
ALTER TABLE accounts ADD COLUMN note text;
DROP TABLE accounts;
SELECT * FROM orders;
