SELECT id FROM orders WHERE candidate_id = ANY($1::uuid[]); -- no-mistakes-bound postgres-bounded-statements rows<=50: page of ids
