-- An inner zero-row cap survives a later ORDER BY set-returning expansion.
(SELECT count(*) FROM orders WHERE false LIMIT 0) ORDER BY generate_series(1, 1000000);
(SELECT count(*) FROM orders LIMIT 0) ORDER BY generate_series(1, 1000000);
((SELECT count(*) FROM orders LIMIT 0)) ORDER BY generate_series(1, 1000000);
(SELECT * FROM orders LIMIT 0) ORDER BY get_all_ids();
(SELECT count(*) FROM orders FETCH FIRST 0 ROWS ONLY) ORDER BY generate_series(1, 1000000);
-- Positive, NULL and absent inner caps do not prove the expanded output empty.
(SELECT count(*) FROM orders WHERE false LIMIT 1) ORDER BY generate_series(1, 1000000);
(SELECT count(*) FROM orders WHERE false LIMIT NULL) ORDER BY generate_series(1, 1000000);
(SELECT count(*) FROM orders WHERE false) ORDER BY generate_series(1, 1000000);
