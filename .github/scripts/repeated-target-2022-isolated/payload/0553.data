-- Mixed migration: sqlparser rejects DO $$ and PostgreSQL 18 VIRTUAL
-- generated columns. Schema extraction must skip the DO block and still
-- collect the CREATE TABLE generated column.
DO $$ BEGIN
  CREATE TYPE item_kinds AS ENUM ('note');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

SELECT REPLACE('a_b', '_', chr(92) || '_');

CREATE TABLE items (
  id uuid PRIMARY KEY DEFAULT uuidv7(),
  created_at timestamptz GENERATED ALWAYS AS (uuid_extract_timestamp(id)) VIRTUAL,
  note text
);
