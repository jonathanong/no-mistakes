SELECT COALESCE(CAST($1 AS app.value ARRAY), ARRAY[]::uuid[]);
