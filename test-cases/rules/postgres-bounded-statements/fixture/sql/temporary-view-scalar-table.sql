-- Scalar TABLE sources affect view lifetime even when they do not constrain rows.
CREATE TEMP TABLE helper (id integer);
CREATE VIEW orders AS SELECT (TABLE helper LIMIT 1);
SELECT * FROM orders;
DROP TABLE helper CASCADE;
SELECT * FROM orders;

-- Unquoted names cannot drop a case-sensitive quoted source.
CREATE TEMP TABLE "Helper" (id integer);
CREATE VIEW orders AS SELECT (TABLE "Helper" LIMIT 1);
SELECT * FROM orders;
DROP TABLE IF EXISTS helper CASCADE;
SELECT * FROM orders;
DROP TABLE "Helper" CASCADE;
SELECT * FROM orders;

CREATE TEMP TABLE "Mixed.Helper" (id integer);
CREATE VIEW orders AS SELECT (TABLE pg_temp."Mixed.Helper" LIMIT 1);
SELECT * FROM orders;
DROP TABLE pg_temp."Mixed.Helper" CASCADE;
SELECT * FROM orders;

-- A scalar permanent intermediary participates in the physical cascade graph.
CREATE VIEW public.middle AS SELECT (TABLE public.accounts LIMIT 1);
CREATE TEMP VIEW orders AS SELECT * FROM public.middle;
SELECT * FROM orders;
DROP TABLE public.accounts CASCADE;
SELECT * FROM orders;

-- TABLE can read a visible CTE; that alias is not a physical dependency.
CREATE TEMP TABLE helper (id integer);
CREATE VIEW public.middle AS WITH helper AS (SELECT 1 AS id) SELECT (TABLE helper LIMIT 1);
CREATE TEMP VIEW orders AS SELECT * FROM public.middle;
SELECT * FROM orders;
DROP TABLE helper CASCADE;
SELECT * FROM orders;
DROP VIEW public.middle CASCADE;
SELECT * FROM orders;
