-- VERBOSE is only a modifier when unquoted and leading.
ANALYZE VERBOSE a, public.b;
ANALYZE a(col), b;
ANALYZE VERBOSE public.a(first_col, "Mixed"), "Schema"."Table"("Quoted"), c(last_col);
ANALYZE "VERBOSE"(id), public."VERBOSE";
ANALYZE VERBOSE single_table(id);
ANALYZE VERBOSE;
