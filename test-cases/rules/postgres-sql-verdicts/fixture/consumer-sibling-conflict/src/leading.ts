import { query } from '@example/db'

// A leading failed chunk must not hide a later broken ON CONFLICT clause.
query("SELECT (; INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO UPDATE SET id = (")
query("SELECT (; INSERT INTO items (id) SELECT input.id FROM unnest(ARRAY[gen_random_uuid(), gen_random_uuid()]) AS input(id) ON CONFLICT DO NOTHING RETURNING (")
