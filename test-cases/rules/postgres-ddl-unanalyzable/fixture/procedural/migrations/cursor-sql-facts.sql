CREATE TABLE cursor_fact_rows (id integer PRIMARY KEY, score integer NOT NULL);

DO $$
DECLARE
  c CURSOR FOR SELECT abs(-1) AS "loop";
  declared_rows CURSOR FOR SELECT abs(score) FROM cursor_fact_rows;
  opened_rows refcursor;
  row_value record;
BEGIN
  OPEN c;
  OPEN opened_rows FOR SELECT abs(score) FROM cursor_fact_rows;
  FETCH NEXT FROM c INTO row_value;
  CLOSE c;
  OPEN declared_rows;
  FETCH NEXT FROM declared_rows INTO row_value;
  CLOSE declared_rows;
  FOR row_value IN SELECT abs(score) FROM cursor_fact_rows LOOP
    LOOP
      EXIT;
    END LOOP;
  END LOOP;
END
$$;

CREATE FUNCTION cursor_sql_facts() RETURNS void LANGUAGE plpgsql AS $function$
DECLARE
  c CURSOR FOR SELECT abs(-1) AS "loop";
  declared_rows CURSOR FOR SELECT abs(score) FROM cursor_fact_rows;
  opened_rows refcursor;
  row_value record;
BEGIN
  OPEN c;
  OPEN opened_rows FOR SELECT abs(score) FROM cursor_fact_rows;
  FETCH NEXT FROM c INTO row_value;
  CLOSE c;
  OPEN declared_rows;
  FETCH NEXT FROM declared_rows INTO row_value;
  CLOSE declared_rows;
  FOR row_value IN SELECT abs(score) FROM cursor_fact_rows LOOP
    LOOP
      EXIT;
    END LOOP;
  END LOOP;
END
$function$;
