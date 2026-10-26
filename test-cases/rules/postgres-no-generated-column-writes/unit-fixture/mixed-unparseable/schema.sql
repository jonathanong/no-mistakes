-- Enum DDL in DO and VIRTUAL generated columns must both parse in one file.
-- Still flag DML writes to created_at.
DO $$ BEGIN
  CREATE TYPE item_kinds AS ENUM ('note');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

CREATE TABLE items (
  id uuid PRIMARY KEY DEFAULT uuidv7(),
  created_at timestamptz GENERATED ALWAYS AS (uuid_extract_timestamp(id)) VIRTUAL,
  note text
);
