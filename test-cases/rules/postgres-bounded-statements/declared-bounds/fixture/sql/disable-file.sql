-- no-mistakes-disable-file postgres-bounded-statements
-- no-mistakes-bound postgres-bounded-statements rows<=100
SELECT id FROM orders WHERE candidate_id = ANY($1::uuid[]);
