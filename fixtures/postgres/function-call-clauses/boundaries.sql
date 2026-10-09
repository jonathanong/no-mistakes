-- Window partition/frame expressions are outside the supported clause vocabulary.
SELECT sum(1) OVER (PARTITION BY probe_inline_partition() ORDER BY probe_inline_order() ROWS BETWEEN probe_inline_start() PRECEDING AND probe_inline_end() FOLLOWING);
SELECT sum(1) OVER w FROM orders WINDOW w AS (PARTITION BY probe_named_partition() ORDER BY probe_named_order() ROWS BETWEEN probe_named_start() PRECEDING AND probe_named_end() FOLLOWING);
SELECT sum(1) OVER (ROWS UNBOUNDED PRECEDING), sum(1) OVER (ROWS BETWEEN CURRENT ROW AND UNBOUNDED FOLLOWING);
-- Synthetic SELECT wrappers must not turn procedural expressions into real SELECT lists.
DO $$ BEGIN IF probe_if() THEN PERFORM probe_perform(); END IF; RETURN probe_return(); END $$;
DO $$ BEGIN IF probe_guard() THEN SELECT probe_real_select() WHERE probe_real_where(); END IF; END $$;
DO $$ BEGIN RETURN QUERY SELECT probe_return_query(); END $$;
DO $$ BEGIN PERFORM (SELECT probe_nested_real_select() WHERE probe_nested_real_where()); END $$;
