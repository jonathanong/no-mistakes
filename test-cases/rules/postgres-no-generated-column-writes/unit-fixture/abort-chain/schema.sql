CREATE TABLE orders (id int, computed int GENERATED ALWAYS AS (id + 1) STORED);
BEGIN;
ALTER TABLE orders ADD COLUMN discarded int GENERATED ALWAYS AS (id + 2) STORED;
ABORT WORK AND CHAIN;
-- ABORT starts another transaction: this drop must be rolled back too.
DROP TABLE orders;
ROLLBACK;

CREATE TABLE bare_orders (id int, computed int GENERATED ALWAYS AS (id + 1) STORED);
BEGIN;
ABORT AND CHAIN;
DROP TABLE bare_orders;
ROLLBACK;

CREATE TABLE transaction_orders (id int, computed int GENERATED ALWAYS AS (id + 1) STORED);
BEGIN;
ABORT TRANSACTION AND CHAIN;
DROP TABLE transaction_orders;
ROLLBACK;

CREATE TABLE no_chain_orders (id int, computed int GENERATED ALWAYS AS (id + 1) STORED);
BEGIN;
ABORT AND NO CHAIN;
DROP TABLE no_chain_orders;
ROLLBACK;
