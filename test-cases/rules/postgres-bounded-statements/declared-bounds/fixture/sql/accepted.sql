-- no-mistakes-bound postgres-bounded-statements rows<=100: caller passes one page of ids
SELECT c.id, o.id
FROM candidates c
JOIN orders o ON o.candidate_id = c.id
WHERE c.id = ANY($1::uuid[]);
