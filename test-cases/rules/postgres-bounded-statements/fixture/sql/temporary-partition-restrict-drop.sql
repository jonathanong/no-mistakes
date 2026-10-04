-- PostgreSQL rejects both RESTRICT drops because a temporary view depends on
-- the partition child. A multi-target failure must preserve the unrelated temp
-- table too; the later CASCADE exposes both physical namesakes.
CREATE TABLE orders(id integer);
CREATE TABLE spare(id integer);
CREATE TEMP TABLE accounts(id integer) PARTITION BY RANGE(id);
CREATE TEMP TABLE orders PARTITION OF accounts FOR VALUES FROM (0) TO (10);
CREATE TEMP TABLE spare(id integer);
CREATE TEMP VIEW orders_view AS SELECT * FROM orders;
DROP TABLE accounts;
SELECT * FROM orders;
DROP TABLE accounts, spare;
SELECT * FROM spare;
DROP TABLE accounts, spare CASCADE;
SELECT * FROM orders;
SELECT * FROM spare;
