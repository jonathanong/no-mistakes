import { query } from '@example/db';
query('SELECT id FROM topic_posts WHERE topic_id = $1 LIMIT 1', []);
