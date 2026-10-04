-- PostgreSQL warns on a repeated BEGIN; it retains the original rollback boundary.
BEGIN;
SAVEPOINT before_temp;
CREATE TEMP TABLE accounts(id uuid);
BEGIN;
ROLLBACK TO before_temp;
SELECT * FROM accounts;
CREATE TEMP TABLE accounts(id uuid);
BEGIN;
ROLLBACK;
SELECT * FROM accounts;
-- Repeating BEGIN also preserves temporary identities created before the transaction.
CREATE TEMP TABLE accounts(id uuid);
BEGIN;
CREATE TEMP TABLE orders(id uuid);
BEGIN;
ROLLBACK;
SELECT * FROM accounts;
SELECT * FROM orders;
DISCARD TEMP;
