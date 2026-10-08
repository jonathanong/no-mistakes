INSERT INTO foo (id, values) VALUES (1, ARRAY[1, 0])
ON CONFLICT (id) DO UPDATE SET values[values[1]] = 2;
INSERT INTO foo (id, slug) VALUES (1, 'a')
ON CONFLICT (lower(slug)) DO UPDATE SET slug = EXCLUDED.slug;
INSERT INTO foo (id, values) VALUES (1, ARRAY[1, 0])
ON CONFLICT (id) DO UPDATE SET values[1] = 2;
-- Quotes, comments and neighboring statements must retain their source identity.
INSERT INTO "App"."Foo" ("ID", "Values", slug) VALUES (1, ARRAY[1, 2], 'a')
ON CONFLICT ("App".lower(slug), "ID") WHERE "ID" > 0
DO UPDATE SET "Values" /* target comment */ [coalesce("Values"[1], 2)][3] = 4,
slug = EXCLUDED.slug, info.values[1] = 2 WHERE "Foo"."ID" > 0;
SELECT 42;
