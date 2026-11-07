import { query } from '@example/db';
query('SELECT * FROM topic_posts WHERE topic_id = $1 ORDER BY id LIMIT $2', []);
query('SELECT * FROM topic_posts WHERE id = $1 LIMIT 5', []);
