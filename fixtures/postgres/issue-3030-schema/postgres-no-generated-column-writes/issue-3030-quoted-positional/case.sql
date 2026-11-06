CREATE TABLE items(note text, "Normalized" text GENERATED ALWAYS AS(lower(note)) STORED);
INSERT INTO items VALUES ('x','manual');
