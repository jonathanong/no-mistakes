SELECT proname, bool_or(proretset) FROM pg_proc WHERE pronamespace = 'pg_catalog'::regnamespace GROUP BY proname ORDER BY proname;
