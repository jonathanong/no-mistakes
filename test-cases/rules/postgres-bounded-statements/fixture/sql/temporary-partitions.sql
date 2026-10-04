-- PostgreSQL requires both parent and child of a partition tree to be temporary.
CREATE TEMP TABLE accounts(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE orders PARTITION OF accounts FOR VALUES FROM (0) TO (10);
SELECT * FROM orders;
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
-- Detach preserves the partition and views over it when the old parent drops.
CREATE TEMP TABLE accounts(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE orders PARTITION OF accounts FOR VALUES FROM (0) TO (10);
CREATE TEMP VIEW order_lines AS SELECT * FROM orders;
ALTER TABLE accounts DETACH PARTITION orders;
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
SELECT * FROM order_lines;
-- Reattaching makes the child dependent on the parent again.
CREATE TEMP TABLE accounts(id integer) PARTITION BY RANGE (id);
ALTER TABLE accounts ATTACH PARTITION orders FOR VALUES FROM (0) TO (10);
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
