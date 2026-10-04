-- IF NOT EXISTS leaves an already-standalone temporary child untouched.
CREATE TABLE orders(id integer);
CREATE TEMP TABLE accounts(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE orders(id integer);
CREATE TEMP TABLE IF NOT EXISTS orders PARTITION OF accounts FOR VALUES FROM (0) TO (10);
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
