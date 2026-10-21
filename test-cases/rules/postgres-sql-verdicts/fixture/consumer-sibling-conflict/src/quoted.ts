import { query } from '@example/db'
declare const flag: boolean

// The doubled quote is why this fixture exists. sqlparser stores the
// unescaped value, so Display turns `'it''s'` into `'it's'`.
// Q1: complete multi-row targetless insert, then a non-conflict failure.
query(flag ? "SELECT 1" : "INSERT INTO items (id) SELECT input.id FROM unnest(ARRAY[gen_random_uuid(), gen_random_uuid()]) AS input(id) WHERE 'it''s' <> '' ON CONFLICT DO NOTHING; VACUUM (")
// Q2: compliant upsert with the same literal, then VACUUM (.
query(flag ? "SELECT 1" : "INSERT INTO items (id) SELECT input.id FROM unnest(ARRAY[gen_random_uuid(), gen_random_uuid()]) AS input(id) WHERE 'it''s' <> '' ORDER BY input.id ON CONFLICT (id) DO NOTHING; VACUUM (")
// E-string: `E'a\'b'` in SQL. The extra backslash is the JavaScript escape.
// Doubling that quote when rendering lets the conflict lexer judge the insert.
query(flag ? "SELECT 1" : "INSERT INTO items (id) SELECT input.id FROM unnest(ARRAY[gen_random_uuid(), gen_random_uuid()]) AS input(id) WHERE E'a\\'b' <> '' ON CONFLICT DO NOTHING; VACUUM (")
