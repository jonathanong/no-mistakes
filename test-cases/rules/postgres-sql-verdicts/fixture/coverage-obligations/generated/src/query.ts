import { query } from "@example/db";
query("INSERT INTO accounts VALUES (1)");
query("INSERT INTO accounts VALUES (1, 2)");
query("INSERT INTO accounts DEFAULT VALUES");
query("INSERT INTO partial_accounts VALUES (1, 2)");
query("MERGE INTO accounts a USING staging s ON a.id = s.id WHEN MATCHED THEN UPDATE SET *");
