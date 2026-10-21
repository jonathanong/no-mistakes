import { query } from '@example/db'
declare const flag: boolean

// Order is the point. A failing chunk before a later conflict must not lend
// that chunk's ON CONFLICT target to the statement that follows it.
// F2: failing targeted conflict, then a complete multi-row targetless insert.
query(flag ? "SELECT 1" : "INSERT INTO items (id) VALUES (gen_random_uuid()) ON CONFLICT (id) DO NOTHING RETURNING (; INSERT INTO items (id) SELECT input.id FROM unnest(ARRAY[gen_random_uuid(), gen_random_uuid()]) AS input(id) ON CONFLICT DO NOTHING")
// F8: failing single-row targetless chunk, then a compliant multi-row upsert.
query(flag ? "SELECT 1" : "INSERT INTO items (id) VALUES (gen_random_uuid()) ON CONFLICT DO NOTHING RETURNING (; INSERT INTO items (id) SELECT input.id FROM unnest(ARRAY[gen_random_uuid(), gen_random_uuid()]) AS input(id) ORDER BY input.id ON CONFLICT (id) DO NOTHING")
// F10: compliant upsert, then a conflict statement that fails inside itself.
query(flag ? "SELECT 1" : "INSERT INTO items (id) SELECT input.id FROM unnest(ARRAY[gen_random_uuid(), gen_random_uuid()]) AS input(id) ORDER BY input.id ON CONFLICT (id) DO NOTHING; INSERT INTO items (id) SELECT input.id FROM unnest(ARRAY[gen_random_uuid(), gen_random_uuid()]) AS input(id) ON CONFLICT DO NOTHING RETURNING (")
// Controls: each statement alone is clear.
query("INSERT INTO items (id) SELECT input.id FROM unnest(ARRAY[gen_random_uuid(), gen_random_uuid()]) AS input(id) ORDER BY input.id ON CONFLICT (id) DO NOTHING")
query("INSERT INTO items (id) VALUES (gen_random_uuid()) ON CONFLICT DO NOTHING")
