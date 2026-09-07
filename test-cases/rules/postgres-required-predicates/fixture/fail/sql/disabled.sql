-- no-mistakes-disable-next-line postgres-required-predicates: reporting query
SELECT id FROM topics WHERE id = $1;
