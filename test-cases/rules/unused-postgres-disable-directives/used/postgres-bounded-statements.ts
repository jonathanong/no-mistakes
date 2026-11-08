import { query } from '@example/db';
query(`
-- no-mistakes-disable-next-line postgres-bounded-statements: intentional exception
SELECT id FROM topic_posts
`, []);
