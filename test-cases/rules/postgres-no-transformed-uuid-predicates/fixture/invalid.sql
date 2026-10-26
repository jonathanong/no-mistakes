SELECT 1;

-- A later invalid statement must remain unanalyzable after a valid statement.
SELECT id FROM accounts WHERE (;
