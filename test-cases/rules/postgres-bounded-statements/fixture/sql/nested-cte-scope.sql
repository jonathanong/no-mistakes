-- Nested WITH names must not hide a parent source that lacks the outer target column.
DELETE FROM accounts a WHERE a.id IN (SELECT COALESCE(id, (WITH currencies AS (SELECT 1) SELECT NULL)) FROM currencies LIMIT 1);
-- A projected CTE column makes leakage dangerous even when projected outputs are known.
DELETE FROM accounts a WHERE a.id IN (SELECT COALESCE(id, (WITH currencies AS (SELECT 1 AS id) SELECT NULL)) FROM currencies LIMIT 1);
-- The real parent table owns code, so this capped subquery stays independent of accounts.
DELETE FROM accounts a WHERE a.id IN (SELECT COALESCE(code, (WITH currencies AS (SELECT 1 AS id) SELECT NULL)) FROM currencies LIMIT 1);
