-- postgres-explicit-columns: a star over a catalog table.
SELECT * FROM orders WHERE id = $1;
-- postgres-bounded-statements: the same read can return every event of that kind.
-- postgres-required-predicates: a read of the partitioned table that does not bound its key.
SELECT id FROM events WHERE kind = 'login';
-- postgres-no-transformed-uuid-predicates: cast the parameter instead of the catalog UUID key.
SELECT id FROM accounts WHERE CAST(id AS text) = $1 LIMIT 1;
