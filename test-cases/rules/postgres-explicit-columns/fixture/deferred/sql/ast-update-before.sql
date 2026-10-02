-- The shared AST helper must tolerate parser variants beyond PostgreSQL syntax.
UPDATE orders FROM accounts SET status = 'x' RETURNING *;
