import { query } from '@example/db'

// The failure is inside the conflict statement, so a recovered prefix is not proof.
query("INSERT INTO items (id) SELECT input.id FROM unnest(ARRAY[gen_random_uuid(), gen_random_uuid()]) AS input(id) ON CONFLICT DO NOTHING RETURNING (")
