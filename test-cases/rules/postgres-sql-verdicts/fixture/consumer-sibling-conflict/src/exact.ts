import { query } from '@example/db'

// Exact calls, not finite variants. A non-conflict sibling that fails to parse
// stays outside the conflict statement whether it comes before or after.
query("INSERT INTO items (id) SELECT input.id FROM unnest(ARRAY[gen_random_uuid(), gen_random_uuid()]) AS input(id) ON CONFLICT DO NOTHING; VACUUM (")
query("VACUUM (; INSERT INTO items (id) SELECT input.id FROM unnest(ARRAY[gen_random_uuid(), gen_random_uuid()]) AS input(id) ON CONFLICT DO NOTHING")
