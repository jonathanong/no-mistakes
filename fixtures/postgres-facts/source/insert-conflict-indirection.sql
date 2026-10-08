-- Compound arbiters retain nested calls, even when the root is not a function.
INSERT INTO foo (slug) VALUES ('a')
ON CONFLICT ((lower(upper(slug)) || '-x')) DO UPDATE SET slug = coalesce(EXCLUDED.slug, lower(upper(slug))), other = CAST((coalesce('a', lower('b'))) AS TEXT),
-- A wildcard argument has no expression span; keep its incompleteness explicit.
wild = row_to_json(foo.*),
-- Unary operands retain nested call argument delimiters.
score = -abs(length(lower(slug)));
INSERT INTO foo (records) VALUES (NULL)
ON CONFLICT (id) DO UPDATE SET records[1].name = 'x',
records[2]."Items" /* field boundary */ [coalesce(records[1].idx, 2)].name = 'y', records."Items".name[1][2].title = 'z', records."Name" = 'field';
INSERT INTO foo (slug, id) VALUES ('a', 1)
ON CONFLICT (lower(slug) text_pattern_ops (siglen = 32), id "Ops"."IntOps", slug) WHERE coalesce(is_ready(lower(slug)), false) DO UPDATE SET slug = EXCLUDED.slug WHERE coalesce(is_ready(upper(slug)), true);
INSERT INTO foo (slug) VALUES ('a')
ON CONFLICT (slug COLLATE "C" "Ops"."TextOps") DO NOTHING;
SELECT 42;
