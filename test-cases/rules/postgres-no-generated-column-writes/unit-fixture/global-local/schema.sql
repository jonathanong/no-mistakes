CREATE GLOBAL TEMPORARY TABLE orders (id int, computed int GENERATED ALWAYS AS (id + 1) STORED);
DROP TABLE orders;
CREATE TABLE orders (id int, computed int);
CREATE LOCAL TEMP TABLE reverse_orders (id int, computed int);
DROP TABLE reverse_orders;
CREATE TABLE reverse_orders (id int, computed int GENERATED ALWAYS AS (id + 1) STORED);
