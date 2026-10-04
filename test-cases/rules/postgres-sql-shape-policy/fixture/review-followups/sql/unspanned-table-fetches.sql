COMMENT ON TABLE public IS 'a TABLE keyword that is not a query';
CREATE TABLE public.decoy_schema (id integer);

SELECT * FROM (
  TABLE public.first_orders
  FETCH FIRST ROW ONLY
) AS first_page;
CREATE TABLE public.decoy_next (id integer);

SELECT * FROM (
  TABLE public.repeated_orders
) AS unbounded_page
CROSS JOIN (
  TABLE public.repeated_orders
  FETCH FIRST ROW ONLY
) AS repeated_page;
