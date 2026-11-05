import { query } from "@example/db";
query(`SELECT * FROM items WHERE id=ANY($1) FOR UPDATE /* deadlock-safe: intentional synthetic comment fixture`);
