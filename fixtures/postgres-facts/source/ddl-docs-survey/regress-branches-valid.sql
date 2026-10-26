-- These forms are valid PostgreSQL syntax and reach the dedicated ALTER recognizer.
ALTER TABLE t ALTER COLUMN c SET (n_distinct = 1, n_distinct_inherited = 2);
ALTER TABLE t RESET (fillfactor, autovacuum_enabled);
ALTER TABLE t ALTER c SET STATISTICS +1;
ALTER TABLE t ATTACH PARTITION p FOR VALUES WITH (MODULUS 2, REMAINDER 0);
ALTER TABLE t DETACH PARTITION p FINALIZE;
