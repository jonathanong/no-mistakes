-- Permanent-only intermediaries are catalog relations, even when a temporary view reads them.
CREATE VIEW public.middle AS SELECT * FROM public.accounts;
CREATE VIEW public.outer_view AS SELECT * FROM public.middle;
SELECT * FROM public.outer_view;
CREATE TEMP VIEW orders AS SELECT * FROM public.outer_view;
SELECT * FROM orders;
DROP TABLE other.accounts CASCADE;
SELECT * FROM orders;
DROP TABLE public.accounts RESTRICT;
SELECT * FROM orders;
DROP VIEW public.middle RESTRICT;
SELECT * FROM orders;
DROP TABLE public.accounts CASCADE;
SELECT * FROM orders;

-- A materialized source participates in the same physical cascade closure.
CREATE MATERIALIZED VIEW public.materialized_source AS SELECT * FROM public.order_lines;
CREATE TEMP VIEW orders AS SELECT * FROM public.materialized_source;
SELECT * FROM orders;
DROP TABLE public.order_lines CASCADE;
SELECT * FROM orders;

-- A schema cascade also follows an intermediate view in another schema.
CREATE VIEW other.middle AS SELECT * FROM public.accounts;
CREATE TEMP VIEW orders AS SELECT * FROM other.middle;
DROP SCHEMA public CASCADE;
SELECT * FROM orders;

-- A bare view declaration has unknown schema ownership; retire conservatively.
CREATE VIEW middle AS SELECT * FROM other.accounts;
CREATE TEMP VIEW orders AS SELECT * FROM middle;
DROP SCHEMA public CASCADE;
SELECT * FROM orders;
