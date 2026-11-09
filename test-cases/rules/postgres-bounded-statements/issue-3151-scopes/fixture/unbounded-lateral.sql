SELECT o.id, x.label FROM owners o
CROSS JOIN LATERAL (SELECT (SELECT k.label FROM kinds k) AS label) x
WHERE o.id = $1;
