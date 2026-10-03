-- TABLE bodies have no sqlparser span; the containing parentheses precede that span.
SELECT * FROM (
  TABLE public.orders
  -- no-mistakes-disable-next-line postgres-sql-shape-policy
  FETCH FIRST ROW ONLY
) AS source;
