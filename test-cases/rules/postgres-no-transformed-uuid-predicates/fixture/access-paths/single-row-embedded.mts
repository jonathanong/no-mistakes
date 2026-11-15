import { query, sql } from 'db';

declare const postId: string;

query(sql`SELECT * FROM single_row_items WHERE ROW(post_id)::uuid_single = ROW(${postId})::uuid_single;`);
query(sql`SELECT * FROM single_row_items WHERE (post_id)::uuid_single = ${postId}::uuid_single;`);
query(`SELECT * FROM single_row_items WHERE ROW(post_id)::uuid_single = ROW(sql_placeholder_0)::uuid_single;`);
