-- no-mistakes-bound postgres-bounded-statements rows<=1: id is the primary key
SELECT id, email FROM candidates WHERE id = $1;
