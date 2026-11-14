import { query } from 'db';
query(`SELECT * FROM items WHERE (post_id, content_sha256, policy_revision)::item_identity = ($1, $2, $3)::item_identity;`);
query(`SELECT * FROM a LEFT JOIN b ON b.link = a.id LEFT JOIN c ON c.link = a.id WHERE num_nonnulls(COALESCE(a.id, b.id), c.id) = 1;`);
query(`SELECT * FROM a LEFT JOIN b ON b.link = a.id WHERE a.id::text = $1;`);
query(`SELECT * FROM a LEFT JOIN b ON b.link = a.id WHERE b.id::text = $1;`);
query(`SELECT * FROM unindexed_items WHERE (post_id, content_sha256, policy_revision)::item_identity = ($1, $2, $3)::item_identity;`);
query(`SELECT * FROM a LEFT JOIN b ON b.link = a.id LEFT JOIN c ON c.link = a.id WHERE a.id::text = $1 AND num_nonnulls(COALESCE(a.id, b.id), c.id) = 1;`);
