-- Plain EXPLAIN plans a TABLE query; ANALYZE executes it.
CREATE TEMP TABLE "Accounts" (id uuid);
EXPLAIN TABLE "Accounts";
EXPLAIN TABLE Accounts;
EXPLAIN VERBOSE TABLE "Accounts";
EXPLAIN ANALYZE TABLE "Accounts";
EXPLAIN ANALYZE TABLE Accounts;
EXPLAIN ANALYZE TABLE public."Accounts";
