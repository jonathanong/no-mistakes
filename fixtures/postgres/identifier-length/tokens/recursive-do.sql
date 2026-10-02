CREATE RECURSIVE VIEW recursive_items (recursive_column, second_column) AS SELECT 1;
DO LANGUAGE plpgsql $$
CREATE TABLE recovered_items (recovered_column int);
CREATE PROCEDURE recovered_proc() LANGUAGE sql AS $proc$ SELECT 1; $proc$;
$$;
DO LANGUAGE -- malformed prefix must not hide later declarations
CREATE RECURSIVE VIEW view_without_columns AS SELECT 1;
CREATE RECURSIVE VIEW malformed_columns (1) AS SELECT 1; -- invalid column token must terminate extraction
