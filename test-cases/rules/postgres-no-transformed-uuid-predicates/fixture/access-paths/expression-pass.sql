SELECT * FROM items WHERE (post_id, content_sha256, policy_revision)::item_identity = ($1, $2, $3)::item_identity;
SELECT * FROM items i WHERE (i.post_id, i.content_sha256, i.policy_revision)::item_identity = ($1, $2, $3)::item_identity;
SELECT * FROM public.items i WHERE (i.post_id, i.content_sha256, i.policy_revision)::item_identity = ($1, $2, $3)::item_identity;
SELECT * FROM items i JOIN unindexed_items u ON u.id = i.id WHERE (i.post_id, i.content_sha256, i.policy_revision)::item_identity = ($1, $2, $3)::item_identity;
SELECT * FROM functions WHERE lower(id::text) = $1;
SELECT * FROM paired_items WHERE (post_id, id)::uuid_pair = ($1, $2)::uuid_pair;
SELECT * FROM items AS "I" WHERE ("I".post_id, "I".content_sha256, "I".policy_revision)::item_identity = ($1, $2, $3)::item_identity;
SELECT * FROM items WHERE ROW(post_id, content_sha256, policy_revision)::item_identity = ROW($1, $2, $3)::item_identity;
SELECT * FROM single_row_items WHERE ROW(post_id)::uuid_single = ROW($1)::uuid_single;
