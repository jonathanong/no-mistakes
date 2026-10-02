-- GROUP BY makes this HAVING threshold a per-group aggregate, not an existence probe.
SELECT account_id FROM orders GROUP BY account_id HAVING COUNT(*) > 0;
