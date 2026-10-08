-- Compound arbiters retain nested calls, even when the root is not a function.
INSERT INTO foo (slug) VALUES ('a')
ON CONFLICT ((lower(upper(slug)) || '-x')) DO UPDATE SET slug = coalesce(EXCLUDED.slug, lower(upper(slug))), other = CAST((coalesce('a', lower('b'))) AS TEXT);
INSERT INTO foo (records) VALUES (NULL)
ON CONFLICT (id) DO UPDATE SET records[1].name = 'x',
records[2]."Items" /* field boundary */ [coalesce(records[1].idx, 2)].name = 'y';
INSERT INTO foo (slug, id) VALUES ('a', 1)
ON CONFLICT (lower(slug) text_pattern_ops, id "Ops"."IntOps", slug) WHERE id > 0 DO NOTHING;
INSERT INTO foo (slug) VALUES ('a')
ON CONFLICT (slug COLLATE "C" "Ops"."TextOps") DO NOTHING;
SELECT 42;
