INSERT INTO a (id) SELECT 1 FROM t WHERE x = ANY(t.aliases) AND NOT EXISTS (SELECT 1 FROM b) ON CONFLICT DO NOTHING;
SELECT 1 FROM "Schéma"."Table" AS "T" WHERE "T"."x" <> ALL("T"."aliases");
SELECT 1 FROM t WHERE -1 < ANY(ARRAY[-1, 2]);
SELECT 1 FROM t WHERE x <= ALL('{1,2}'::integer[]);
SELECT 1 FROM t WHERE 3 > ANY($1::integer[]);
SELECT 1 FROM t WHERE $1 >= ALL(t.aliases) AND +1 > ANY($2) AND (x) = ALL((t.aliases)) AND x = ANY($3) AND DATE 'today' = ANY($4) AND 1 = ANY(NULL) AND 1 = ANY('{1,2}');
SELECT 1 FROM t WHERE 'x' = ANY(CAST($2 AS text[]));
SELECT 1 FROM t JOIN u ON t.x = ANY(u.aliases) WHERE NOT (t.x = ALL(t.aliases)) OR t.x = ANY(t.aliases);
-- Existing predicates and INSERT provenance remain separate from quantified comparisons.
INSERT INTO a (id) SELECT 1 FROM t WHERE t.aliases @> ARRAY['x'] AND x IN (1, 2) AND NOT EXISTS (SELECT 1 FROM b) ON CONFLICT DO NOTHING;
