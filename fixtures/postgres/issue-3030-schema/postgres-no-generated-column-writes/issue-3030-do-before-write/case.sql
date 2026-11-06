CREATE TABLE items(note text, normalized text GENERATED ALWAYS AS(lower(note)) STORED); DO $$ BEGIN DROP TABLE items; END $$; UPDATE items SET normalized='manual';
