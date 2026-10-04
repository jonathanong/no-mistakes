-- A plain DROP owns the attached child even when a persistent namesake exists.
CREATE TABLE orders(id integer);
CREATE TABLE renamed_orders(id integer);
CREATE TEMP TABLE accounts(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE orders PARTITION OF accounts FOR VALUES FROM (0) TO (10);
DROP TABLE accounts;
SELECT * FROM orders;
-- Renaming a parent moves ownership to its new name.
CREATE TEMP TABLE accounts(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE orders PARTITION OF accounts FOR VALUES FROM (0) TO (10);
ALTER TABLE accounts RENAME TO active_accounts;
DROP TABLE active_accounts;
SELECT * FROM orders;
-- Renaming a child moves its ownership key as well.
CREATE TEMP TABLE accounts(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE orders PARTITION OF accounts FOR VALUES FROM (0) TO (10);
ALTER TABLE orders RENAME TO renamed_orders;
DROP TABLE accounts;
SELECT * FROM renamed_orders;
-- The concurrent form detaches the child before the old parent is dropped.
CREATE TEMP TABLE accounts(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE orders PARTITION OF accounts FOR VALUES FROM (0) TO (10);
ALTER TABLE accounts DETACH PARTITION orders CONCURRENTLY;
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
