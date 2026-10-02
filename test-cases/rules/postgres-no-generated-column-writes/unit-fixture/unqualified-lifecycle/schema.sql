SET search_path = public;
CREATE TABLE public.orders (id int);
ALTER TABLE orders ADD COLUMN computed int GENERATED ALWAYS AS (id + 1) STORED;
CREATE TABLE public.removed (id int, computed int GENERATED ALWAYS AS (id + 1) STORED);
DROP TABLE removed;
CREATE TABLE public."odd.name" (id int);
ALTER TABLE "odd.name" ADD COLUMN computed int GENERATED ALWAYS AS (id + 1) STORED;

-- Ambiguous suffixes cannot select either schema without resolving search_path.
CREATE TABLE public.ambiguous (id int);
CREATE TABLE audit.ambiguous (id int);
ALTER TABLE ambiguous ADD COLUMN computed int GENERATED ALWAYS AS (id + 1) STORED;
DROP TABLE ambiguous;
ALTER TABLE public.ambiguous ADD COLUMN retained int GENERATED ALWAYS AS (id + 1) STORED;

-- Temporary tables retain priority when permanent suffixes coexist.
CREATE TABLE public.shadowed (id int);
CREATE TEMP TABLE shadowed (id int);
ALTER TABLE shadowed ADD COLUMN computed int GENERATED ALWAYS AS (id + 1) STORED;
