-- Unicode preceding a same-line body must retain original byte and scalar positions.
/* 雪 */ DO $body$ BEGIN ALTER TABLE "Café" ADD COLUMN "雪" integer; END $body$ LANGUAGE plpgsql;
DO LANGUAGE PLPGSQL $$ BEGIN CREATE TABLE nested_table (id integer); END $$;
DO $$ BEGIN IF true THEN ALTER TABLE hidden_table ADD COLUMN hidden integer; END IF; END $$;
DO $$ BEGIN DECLARE x integer; ALTER TABLE hidden_table ADD COLUMN hidden integer; END $$;
DO $$ SELECT 1; $$;
DO $$ BEGIN ALTER TABLE broken (bad syntax); CREATE TABLE after_error (id integer); END $$;
DO LANGUAGE sql $$ BEGIN CREATE TABLE unknown_language (id integer); END $$;
DO 'BEGIN CREATE TABLE escaped_body (id integer); END';
CREATE TABLE after_procedural (id integer);
