import { query } from '@example/db'
declare const flag: boolean

// The INSERT is complete. Only the following statement fails to parse.
query(flag ? "SELECT 1" : "INSERT INTO items (id) SELECT input.id FROM unnest(ARRAY[gen_random_uuid(), gen_random_uuid()]) AS input(id) ON CONFLICT DO NOTHING; VACUUM (")
