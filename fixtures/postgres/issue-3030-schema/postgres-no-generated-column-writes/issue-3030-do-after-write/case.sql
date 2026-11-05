CREATE TABLE items(note text, normalized text GENERATED ALWAYS AS(lower(note)) STORED); UPDATE items SET normalized='manual'; DO $$ BEGIN DROP TABLE items; END $$;
