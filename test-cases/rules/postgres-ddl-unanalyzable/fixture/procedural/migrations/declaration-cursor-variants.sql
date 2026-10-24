CREATE TABLE cursor_variant_rows (
  id integer PRIMARY KEY,
  label text COLLATE pg_catalog."C" NOT NULL,
  score numeric(10, 2) NOT NULL
);

DO $$
DECLARE
  minimum_score numeric(10, 2) NOT NULL := 1.25;
  label_filter text COLLATE pg_catalog."C" DEFAULT 'ready';
  row_id integer := 1;
  result_label text;
  result_score numeric(10, 2);
  matrix_value integer[][] := ARRAY[[1, 2], [3, 4]];
  slice_value integer[];
  parameterized SCROLL CURSOR (minimum numeric(10, 2), wanted_label text) FOR
    SELECT id, label, score FROM cursor_variant_rows WHERE score >= minimum;
  static_rows CURSOR FOR SELECT id, label, score FROM cursor_variant_rows;
  dynamic_rows refcursor;
BEGIN
  OPEN parameterized(COALESCE(minimum_score, 0.00), lower(label_filter));
  FETCH NEXT FROM parameterized INTO row_id, result_label, result_score;
  MOVE RELATIVE -2 FROM parameterized;
  FETCH RELATIVE -1 FROM parameterized INTO row_id, result_label, result_score;
  CLOSE parameterized;
  OPEN static_rows;
  FETCH NEXT FROM static_rows INTO row_id, result_label, result_score;
  CLOSE static_rows;
  OPEN dynamic_rows NO SCROLL FOR SELECT id, label, score FROM cursor_variant_rows;
  FETCH NEXT FROM dynamic_rows INTO row_id, result_label, result_score;
  CLOSE dynamic_rows;
  OPEN dynamic_rows SCROLL FOR EXECUTE
    'SELECT id, label, score FROM cursor_variant_rows WHERE score >= $1'
    USING minimum_score;
  FETCH NEXT FROM dynamic_rows INTO row_id, result_label, result_score;
  CLOSE dynamic_rows;
  FOR row_id IN REVERSE 10..1 BY 2 LOOP
    NULL;
  END LOOP;
  FOREACH slice_value SLICE 1 IN ARRAY matrix_value LOOP
    NULL;
  END LOOP;
END
$$;

CREATE FUNCTION cursor_variant_function() RETURNS void LANGUAGE plpgsql AS $function$
DECLARE
  minimum_score numeric(10, 2) NOT NULL := 1.25;
  label_filter text COLLATE pg_catalog."C" DEFAULT 'ready';
  row_id integer := 1;
  result_label text;
  result_score numeric(10, 2);
  parameterized SCROLL CURSOR (minimum numeric(10, 2), wanted_label text) FOR
    SELECT id, label, score FROM cursor_variant_rows WHERE score >= minimum;
  dynamic_rows refcursor;
BEGIN
  OPEN parameterized(COALESCE(minimum_score, 0.00), lower(label_filter));
  FETCH NEXT FROM parameterized INTO row_id, result_label, result_score;
  MOVE RELATIVE -2 FROM parameterized;
  FETCH RELATIVE -1 FROM parameterized INTO row_id, result_label, result_score;
  CLOSE parameterized;
  OPEN dynamic_rows NO SCROLL FOR SELECT id, label, score FROM cursor_variant_rows;
  FETCH NEXT FROM dynamic_rows INTO row_id, result_label, result_score;
  CLOSE dynamic_rows;
  OPEN dynamic_rows SCROLL FOR EXECUTE
    'SELECT id, label, score FROM cursor_variant_rows WHERE score >= $1'
    USING minimum_score;
  FETCH NEXT FROM dynamic_rows INTO row_id, result_label, result_score;
  CLOSE dynamic_rows;
  FOR row_id IN REVERSE 10..1 BY 2 LOOP
    NULL;
  END LOOP;
END
$function$;
