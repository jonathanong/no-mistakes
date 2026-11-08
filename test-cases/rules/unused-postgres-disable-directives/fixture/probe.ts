import { query } from '@example/db';
// no-mistakes-disable-next-line postgres-required-predicates: accepted probe
query('SELECT id FROM topic_posts WHERE topic_id = $1 LIMIT 1', []);
query(`
-- no-mistakes-disable-next-line postgres-required-predicates: accepted probe
SELECT id FROM topic_posts WHERE topic_id = $1 LIMIT 1
`, []);
// no-mistakes-disable-next-line postgres-bounded-statements: accepted probe
query('SELECT id FROM topic_posts WHERE topic_id = $1 LIMIT 1', []);
query(`
-- no-mistakes-disable-next-line postgres-bounded-statements: accepted probe
SELECT id FROM topic_posts WHERE topic_id = $1 LIMIT 1
`, []);
// no-mistakes-disable-next-line postgres-no-transformed-uuid-predicates: accepted probe
query('SELECT id FROM topic_posts WHERE topic_id = $1 LIMIT 1', []);
query(`
-- no-mistakes-disable-next-line postgres-no-transformed-uuid-predicates: accepted probe
SELECT id FROM topic_posts WHERE topic_id = $1 LIMIT 1
`, []);
// no-mistakes-disable-next-line postgres-lock-ordering: accepted probe
query('SELECT id FROM topic_posts WHERE topic_id = $1 LIMIT 1', []);
query(`
-- no-mistakes-disable-next-line postgres-lock-ordering: accepted probe
SELECT id FROM topic_posts WHERE topic_id = $1 LIMIT 1
`, []);
// no-mistakes-disable-next-line postgres-lock-written-rows-only: accepted probe
query('SELECT id FROM topic_posts WHERE topic_id = $1 LIMIT 1', []);
query(`
-- no-mistakes-disable-next-line postgres-lock-written-rows-only: accepted probe
SELECT id FROM topic_posts WHERE topic_id = $1 LIMIT 1
`, []);
const example = `
-- no-mistakes-disable-next-line postgres-required-predicates: unexecuted example
SELECT id FROM topic_posts
`;
export { example };
