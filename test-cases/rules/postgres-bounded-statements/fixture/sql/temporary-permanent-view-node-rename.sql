-- ALTER TABLE RENAME is also valid PostgreSQL syntax for views.
CREATE VIEW public.middle AS SELECT * FROM public.accounts;
ALTER TABLE public.middle RENAME TO renamed;
CREATE TEMP VIEW orders AS SELECT * FROM public.renamed;
SELECT * FROM orders;
DROP VIEW IF EXISTS public.middle CASCADE;
SELECT * FROM orders;
DROP VIEW IF EXISTS other.renamed CASCADE;
SELECT * FROM orders;
DROP TABLE public.accounts CASCADE;
SELECT * FROM orders;

-- Materialized view nodes must move along with their dependency edges.
CREATE MATERIALIZED VIEW public.middle_mat AS SELECT * FROM public.order_lines;
ALTER TABLE public.middle_mat RENAME TO renamed_mat;
CREATE TEMP VIEW orders AS SELECT * FROM public.renamed_mat;
SELECT * FROM orders;
DROP TABLE public.order_lines CASCADE;
SELECT * FROM orders;

-- An ambiguous bare declaration retains its original identity when another schema is renamed.
CREATE VIEW middle AS SELECT * FROM other.accounts;
ALTER TABLE IF EXISTS other.middle RENAME TO moved;
CREATE TEMP VIEW orders AS SELECT * FROM public.middle;
SELECT * FROM orders;
DROP TABLE other.accounts CASCADE;
SELECT * FROM orders;

-- Quoted relation names keep their exact identity within the original schema.
CREATE TABLE public.accounts (id integer);
CREATE VIEW public."Inner.Name" AS SELECT * FROM public.accounts;
ALTER TABLE public."Inner.Name" RENAME TO "New.Name";
CREATE TEMP VIEW orders AS SELECT * FROM public."New.Name";
SELECT * FROM orders;
DROP TABLE public.accounts CASCADE;
SELECT * FROM orders;

-- A real bare rename must also expose the moved node to qualified dependents.
CREATE TABLE other.accounts (id integer);
CREATE VIEW middle AS SELECT * FROM other.accounts;
ALTER TABLE middle RENAME TO moved;
CREATE TEMP VIEW orders AS SELECT * FROM public.moved;
SELECT * FROM orders;
DROP TABLE other.accounts CASCADE;
SELECT * FROM orders;

-- PostgreSQL rejects this known target collision; keep the original graph unchanged.
CREATE TABLE public.accounts (id integer);
CREATE TABLE public.order_lines (id integer);
CREATE VIEW public.middle AS SELECT * FROM public.accounts;
CREATE VIEW public.taken AS SELECT * FROM public.order_lines;
CREATE TEMP VIEW orders AS SELECT * FROM public.middle;
SELECT * FROM orders;
ALTER TABLE public.middle RENAME TO taken;
DROP VIEW public.middle CASCADE;
SELECT * FROM orders;
