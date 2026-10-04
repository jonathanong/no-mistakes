CREATE TABLE variants (
 id int NULL,
 name text UNIQUE,
 parent uuid REFERENCES parents(id),
 positive numeric CHECK (positive > 0),
 collated text COLLATE "C",
 generated text GENERATED ALWAYS AS (lower(name)) STORED,
 precise numeric(8),
 exact numeric(12,3),
 broad numeric,
 decimal_value decimal(9,2),
 dec_value dec(7),
 short varchar(20),
 chars character varying(10),
 fixed char(2),
 long_fixed character(4),
 instant timestamp(3),
 clock time(4),
 array_values integer ARRAY[4],
 custom app.money(12,2)
);
ALTER TABLE variants RENAME COLUMN name TO title;
ALTER TABLE variants ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY;
CREATE FUNCTION table_result() RETURNS TABLE (id int, name text) LANGUAGE sql AS $$ SELECT 1, 'x' $$;
ALTER TABLE variants VALIDATE CONSTRAINT positive_check;
