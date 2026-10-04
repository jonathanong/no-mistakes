-- A plain parent drop removes an attached temp child, revealing its catalog namesake.
CREATE TABLE orders(id integer);
CREATE TEMP TABLE accounts(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE orders PARTITION OF accounts FOR VALUES FROM (0) TO (10);
DROP TABLE accounts;
SELECT * FROM orders;
