INSERT INTO t AS t (v) VALUES(1) ON CONFLICT(v) DO UPDATE SET
  quoted_date = 'now'::"date",
  qualified_date = 'now'::"schema.with.dot".date,
  nested_date = ('now'::"schema.with.dot".date)::"date",
  and_predicate = t.v IS NULL AND EXCLUDED.v IS NOT NULL,
  or_predicate = t.v IS NULL OR EXCLUDED.v IS NOT NULL,
  cast_predicate = (t.v::"schema.with.dot".date) IS NULL,
  -- Unsupported subquery structure must continue to fail closed.
  opaque_expression = (SELECT v FROM t)
WHERE t.v IS NULL AND EXCLUDED.v IS NOT NULL;
