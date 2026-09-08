import { query } from "@data-stores/psql";

declare const tableName: string;

query(`SELECT *
FROM ${tableName}
-- INSERT INTO items ON CONFLICT (id) DO NOTHING
WHERE note = 'INSERT INTO items ON CONFLICT (id) DO NOTHING'`);
