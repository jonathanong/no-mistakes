SELECT 1 FROM (accounts a JOIN orders o ON a.id::text = $1);
SELECT 1 FROM accounts a JOIN (orders o JOIN accounts b ON o.account_id::text = b.id::text) ON a.id = o.account_id;
SELECT 1 FROM (accounts a JOIN orders o ON a.id::text = $1) joined;
SELECT 1 FROM (accounts a JOIN (orders o JOIN accounts b ON o.id::text = $1) ON a.id = b.id) WHERE a.id::text = $1;
SELECT 1 FROM (accounts a JOIN orders o ON lower($1) = lower($2)) WHERE o.id = $1::uuid;
