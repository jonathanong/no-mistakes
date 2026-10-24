SELECT 1;

-- Preserve uncertainty even when the parse failure follows a valid statement.
SELECT id FROM users WHERE (
