import { query } from '@example/db';
query(`
-- no-mistakes-disable-next-line postgres-required-predicates: intentional exception
SELECT id FROM topic_posts
`, []);
