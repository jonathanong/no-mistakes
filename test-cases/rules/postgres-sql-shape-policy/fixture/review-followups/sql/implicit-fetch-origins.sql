-- Each TABLE body and implicit FETCH has only partial AST spans.
SELECT * FROM (
  TABLE public.orders
  FETCH FIRST ROW ONLY
) AS first_result;

SELECT * FROM (
  TABLE public.accounts
  -- no-mistakes-disable-next-line postgres-sql-shape-policy
  FETCH FIRST ROW ONLY
) AS second_result;

SELECT * FROM (
  (SELECT id FROM inner_rows FETCH FIRST ROW ONLY)
  UNION ALL
  TABLE public.audit
  -- no-mistakes-disable-next-line postgres-sql-shape-policy
  FETCH FIRST ROW ONLY
) AS set_result;

WITH first_page AS (
  TABLE public.first_rows
  FETCH FIRST ROW ONLY
), second_page AS (
  TABLE public.second_rows
  FETCH FIRST ROW ONLY
)
SELECT * FROM first_page UNION ALL SELECT * FROM second_page;

SELECT * FROM (
  (TABLE public.left_rows FETCH FIRST ROW ONLY)
  UNION ALL
  (TABLE public.right_rows FETCH FIRST ROW ONLY)
) AS table_union;

TABLE public.standalone_first FETCH FIRST ROW ONLY;
TABLE public.standalone_second
-- no-mistakes-disable-next-line postgres-sql-shape-policy
FETCH FIRST ROW ONLY;
