CREATE TABLE grammar_option_rows (id integer PRIMARY KEY, label text NOT NULL);

DO $$
DECLARE
  matrix_value integer[][] := ARRAY[[1, 2], [3, 4]];
  lower_bound integer := 1;
  upper_bound integer := 3;
  step_value integer := 1;
  range_value integer;
  selected_id integer;
  row_value record;
  selected_rows NO SCROLL CURSOR FOR
    WITH chosen AS (SELECT id FROM grammar_option_rows)
    SELECT id FROM chosen;
BEGIN
  BEGIN DECLARE
    inner_value integer := 1;
    other_value integer DEFAULT 2;
  BEGIN
    inner_value := other_value;
  END; END;
  <<inner>> DECLARE
    labeled_value integer := 1;
  BEGIN
    labeled_value = 2;
  END;
  matrix_value[1][1] := 5;
  OPEN selected_rows;
  FETCH NEXT FROM selected_rows INTO selected_id;
  CLOSE selected_rows;
  FOR range_value IN lower_bound .. upper_bound BY step_value LOOP
    NULL;
  END LOOP;
  FOR row_value IN
    WITH chosen AS (SELECT id FROM grammar_option_rows)
    SELECT id FROM chosen
  LOOP
    selected_id := row_value.id;
  END LOOP;
END
$$;

CREATE FUNCTION grammar_options(input_value integer) RETURNS void LANGUAGE plpgsql AS $function$
DECLARE
  input_alias ALIAS FOR $1;
  matrix_value integer[][] := ARRAY[[1, 2], [3, 4]];
  lower_bound integer := 1;
  upper_bound integer := 3;
  step_value integer := 1;
  range_value integer;
  selected_id integer;
  row_value record;
  selected_rows NO SCROLL CURSOR FOR
    WITH chosen AS (SELECT id FROM grammar_option_rows)
    SELECT id FROM chosen;
BEGIN
  matrix_value[1][1] := input_alias;
  OPEN selected_rows;
  FETCH NEXT FROM selected_rows INTO selected_id;
  CLOSE selected_rows;
  FOR range_value IN lower_bound .. upper_bound BY step_value LOOP
    NULL;
  END LOOP;
  FOR row_value IN
    WITH chosen AS (SELECT id FROM grammar_option_rows)
    SELECT id FROM chosen
  LOOP
    selected_id := row_value.id;
  END LOOP;
  FOR row_value IN
    WITH inserted AS (
      INSERT INTO grammar_option_rows (id, label)
      VALUES (4, 'from cte')
      RETURNING id
    )
    SELECT id FROM inserted
  LOOP
    selected_id := row_value.id;
  END LOOP;
  FOR row_value IN
    WITH seed AS (SELECT 5 AS id)
    INSERT INTO grammar_option_rows (id, label)
    SELECT id, 'with dml' FROM seed RETURNING id
  LOOP
    selected_id := row_value.id;
  END LOOP;
END
$function$;
