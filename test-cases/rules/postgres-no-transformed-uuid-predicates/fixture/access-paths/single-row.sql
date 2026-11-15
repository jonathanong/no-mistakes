SELECT * FROM single_row_items WHERE ROW(post_id)::uuid_single = ROW($1)::uuid_single;
SELECT * FROM single_row_items WHERE (post_id)::uuid_single = $1::uuid_single;
SELECT * FROM single_row_items WHERE ROW(post_id)::uuid_single = ROW(sql_placeholder_0)::uuid_single;
