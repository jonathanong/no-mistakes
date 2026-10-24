DO $branch_body$
BEGIN
  IF
    abs(-1) > 0 AND (CASE WHEN TRUE THEN TRUE ELSE FALSE END)
  THEN
    CREATE RECURSIVE VIEW do_if_head AS SELECT 1;
  ELSIF FALSE THEN
    CREATE RECURSIVE VIEW do_elsif_head AS SELECT 1;
  ELSE
    CREATE RECURSIVE VIEW do_else_head AS SELECT 1;
  END IF;

  <<do_loop>>
  LOOP
    CREATE RECURSIVE VIEW do_loop_head AS SELECT 1;
    IF TRUE THEN
      CREATE RECURSIVE VIEW do_nested_loop_if_head AS SELECT 1;
    END IF;
    EXIT do_loop;
  END LOOP do_loop;

  CASE TRUE
    WHEN TRUE THEN
      CREATE RECURSIVE VIEW do_case_when_head AS SELECT 1;
    ELSE
      CREATE RECURSIVE VIEW do_case_else_head AS SELECT 1;
  END CASE;

  CREATE RECURSIVE VIEW do_valid (id) AS SELECT 1;
  FROBULATE 0b101;
END
$branch_body$;

CREATE FUNCTION branch_body_heads() RETURNS void LANGUAGE plpgsql AS $function_body$
BEGIN
  IF
    abs(-2) > 0 AND (CASE WHEN TRUE THEN TRUE ELSE FALSE END)
  THEN
    CREATE RECURSIVE VIEW function_if_head AS SELECT 1;
  ELSIF FALSE THEN
    CREATE RECURSIVE VIEW function_elsif_head AS SELECT 1;
  ELSE
    CREATE RECURSIVE VIEW function_else_head AS SELECT 1;
  END IF;

  <<function_loop>>
  LOOP
    CREATE RECURSIVE VIEW function_loop_head AS SELECT 1;
    IF TRUE THEN
      CREATE RECURSIVE VIEW function_nested_loop_if_head AS SELECT 1;
    END IF;
    EXIT function_loop;
  END LOOP function_loop;

  CASE TRUE
    WHEN TRUE THEN
      CREATE RECURSIVE VIEW function_case_when_head AS SELECT 1;
    ELSE
      CREATE RECURSIVE VIEW function_case_else_head AS SELECT 1;
  END CASE;

  CREATE RECURSIVE VIEW function_valid (id) AS SELECT 1;
  FROBULATE 0x1F;
END
$function_body$;

-- A real top-level sibling after the routine body remains independently reportable.
CREATE RECURSIVE VIEW top_level_after_body AS SELECT 0x1F;
