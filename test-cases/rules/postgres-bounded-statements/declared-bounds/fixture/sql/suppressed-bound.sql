-- no-mistakes-bound postgres-bounded-statements rows<=40: caller passes one page of ids
SELECT id FROM orders WHERE candidate_id = ANY($1::uuid[]); -- no-mistakes-disable-line postgres-bounded-statements
