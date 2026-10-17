CREATE VIEW first_view AS SELECT (TABLE "Helper" LIMIT 1);
CREATE VIEW second_view AS SELECT (TABLE pg_temp."Mixed.Helper" LIMIT 1);
CREATE VIEW third_view AS WITH helper AS (SELECT 1 AS id) SELECT (TABLE helper LIMIT 1);
