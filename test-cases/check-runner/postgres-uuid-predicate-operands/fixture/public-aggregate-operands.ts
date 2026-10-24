import { query } from '@db';

// Match the raw SQL cases through configured executor literals at both efforts.
query('SELECT id FROM users GROUP BY id HAVING COUNT(DISTINCT id) >= 2');
query('SELECT id FROM users GROUP BY id HAVING MIN(id) >= $1');
query('SELECT id FROM users GROUP BY id HAVING MAX(id) < $1');
query('SELECT count(*) FROM users HAVING COUNT(id::text) >= 2');
query('SELECT count(*) FROM users HAVING cardinality(ARRAY_AGG(id)) >= 2');
query('SELECT count(*) FROM users HAVING lower(MIN(id)::text) = $1');
query('SELECT id FROM users WHERE id >= fn_min_child_uuidv7($1)');
query('SELECT id FROM users WHERE fn_min_child_uuidv7($1) <= id');
query('SELECT id FROM users WHERE id >= fn_min_child_uuidv7(email)');
query('SELECT u.id FROM users u JOIN sessions s ON u.id >= fn_min_child_uuidv7($1)');
query('SELECT id FROM users WHERE uuid_extract_timestamp(id) >= $1');
query('SELECT id FROM users WHERE id::text = $1');
query('SELECT id FROM users WHERE id >= fn_min_child_uuidv7(external_ref)');
query('SELECT count(*) FROM users HAVING count(id) FILTER (WHERE id::text = $1) >= 2');
query('SELECT u.id FROM users u JOIN sessions s ON uuid_extract_timestamp(u.id) >= $1');
query('SELECT id FROM users GROUP BY id HAVING uuid_extract_timestamp(id) >= $1');
