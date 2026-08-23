CREATE TABLE public.table_level_pk (
  id uuid NOT NULL DEFAULT uuidv7(),
  name text,
  PRIMARY KEY (id)
);
