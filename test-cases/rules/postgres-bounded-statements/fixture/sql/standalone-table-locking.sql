-- Locking clauses are valid after a standalone TABLE relation.
TABLE accounts FOR UPDATE;
TABLE accounts FOR SHARE;
