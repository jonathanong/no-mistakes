import { query } from '@example/db';
query(`
-- no-mistakes-disable-next-line postgres-no-transformed-uuid-predicates: intentional exception
SELECT id FROM topic_posts WHERE id::text = $1
`, []);
