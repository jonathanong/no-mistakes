-- sqlparser accepts the AST, but PostgreSQL rejects an integer as an AND operand.
-- Keep this semantic-invalid control to prove malformed booleans stay opaque.
DELETE FROM accounts WHERE enabled = ANY(ARRAY[1 AND TRUE]);
