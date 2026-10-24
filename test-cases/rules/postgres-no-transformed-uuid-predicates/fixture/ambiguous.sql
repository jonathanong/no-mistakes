SELECT 1 FROM accounts a JOIN orders o ON true WHERE id::text = $1;
