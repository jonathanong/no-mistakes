import { query } from '@example/db';
import { probe } from './helper.mjs';
declare const choose: boolean;
declare const unknownSql: string;
query(probe(), []);
query(probe(), []);
query(choose ? `
-- no-mistakes-disable-next-line postgres-required-predicates: first branch
SELECT id FROM topic_posts WHERE topic_id = $1 LIMIT 1
` : `
-- no-mistakes-disable-next-line postgres-required-predicates: second branch
SELECT id FROM topic_posts WHERE topic_id = $1 LIMIT 2
`, []);
query('SELECT id FROM topic_posts WHERE topic_id = $1 LIMIT 1', []);
query(unknownSql, []);
