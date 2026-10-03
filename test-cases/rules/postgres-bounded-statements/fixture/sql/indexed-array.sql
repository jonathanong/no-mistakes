-- Scalar indexes preserve the source dependency; slices retain unbounded stored-array size.
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.id=ANY(ARRAY[o.account_ids[1]]);
UPDATE accounts a SET name='x' FROM orders o WHERE a.id=ANY(ARRAY[o.account_ids[1]]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.id=ANY(ARRAY[o.account_ids[1:2]]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.id=ANY(ARRAY[o.account_ids[$2]]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.id=ANY(ARRAY[o.matrix_ids[1][2]]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.id=ANY(ARRAY[o.domain_ids[1]]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.id=ANY(ARRAY[o.unknown_array[1]]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.id=ANY(ARRAY[o.trailing_array[1]]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.id=ANY(ARRAY[o.malformed_array[1]]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.id=ANY(ARRAY[o.account_id[1]]);
-- Parenthesized column roots and expression indexes keep canonical owner references.
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.id=ANY(ARRAY[(o.account_ids)[1]]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.id=ANY(ARRAY[coalesce(o.account_ids[1], $2)]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.id=ANY(ARRAY[(o.account_ids || o.account_ids)[1]]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.id=ANY(ARRAY[(o.account_ids).field]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.id=ANY(ARRAY[unknown.account_ids[1]]);
DELETE FROM orders WHERE id=$1 AND account_id=ANY(ARRAY[account_ids[1]]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.id=ANY(ARRAY[o.malformed_account_id[1]]);
-- Scalar equality retains ordinary dependency traversal for complex indexed expressions.
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.id=(o.account_ids || o.account_ids)[1];
