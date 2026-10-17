CREATE VIEW app.one AS TABLE "App"."Orders";
CREATE VIEW app.many AS TABLE app.orders UNION ALL TABLE app.orders;
CREATE VIEW app.cte AS WITH rows AS (SELECT * FROM app.orders) SELECT * FROM rows;
-- The parser loses quoting in TABLE nodes; competing spellings require a diagnostic.
CREATE VIEW app.ambiguous AS TABLE app."Orders" UNION ALL TABLE app.Orders;
