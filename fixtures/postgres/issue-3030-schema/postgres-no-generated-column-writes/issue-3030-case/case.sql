CREATE TABLE items(note text, normalized text GENERATED ALWAYS AS(lower(note)) STORED, "Normalized" text);
UPDATE items SET "Normalized"='manual';
UPDATE items SET normalized='manual';
