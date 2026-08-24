DO U&'BEGIN !000a -- !D83D!DE00 C:\temp!000a !0043REATE TABLE custom_escape (id uuid); END' UESCAPE '!';

DO 'BEGIN;'
'CREATE TABLE concatenated_boundary (id uuid); END';

DO $$
BEGIN
  EXECUTE pg_catalog.format('CREATE TABLE %I (id uuid)', table_name);
END
$$;

DO 'BEGIN; -- '
'CREATE TABLE inert_table (id uuid); END';

DO E'BEGIN\nRAISE NOTICE ''x'';\nCREATE TABLE escaped_same_line (id uuid);\n
CREATE TABLE physical_next_line (id uuid); END';

DO U&'BEGIN \000a -- \D83D\DE00\000a CREATE TABLE default_surrogate (id uuid); END';

ALTER TABLE surrounding_posts ADD COLUMN visible text;
DO $$ BEGIN
  ALTER TABLE dollar_posts ADD COLUMN dollar_visible text;
END $$;
