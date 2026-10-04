SELECT 1 UNION ALL TABLE "Order Items";
SELECT 1 UNION ALL TABLE public."Order Items";
WITH "Order Items" AS (SELECT 1) SELECT 1 UNION ALL TABLE "Order Items";
SELECT 1 UNION ALL TABLE Accounts; -- Unquoted Accounts folds to accounts; the quoted spelling below preserves uppercase A.
SELECT 1 UNION ALL TABLE "Accounts";
-- Exact TABLE spelling prevents the other CTE spelling from hiding a relation.
WITH "Accounts" AS (SELECT 1) SELECT 1 UNION ALL TABLE Accounts;
WITH accounts AS (SELECT 1) SELECT 1 UNION ALL TABLE "Accounts";
