import { query } from '@example/db';
// no-mistakes-disable-next-line postgres-lock-ordering: intentional exception
query(`
SELECT id FROM topic_posts WHERE id = ANY($1) FOR UPDATE
`, []);
