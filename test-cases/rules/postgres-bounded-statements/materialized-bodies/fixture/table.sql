SELECT (TABLE "Orders") FROM (TABLE accounts) source LIMIT 1;
SELECT (TABLE "Orders" LIMIT 1) FROM (TABLE accounts LIMIT 1) source LIMIT 1;
