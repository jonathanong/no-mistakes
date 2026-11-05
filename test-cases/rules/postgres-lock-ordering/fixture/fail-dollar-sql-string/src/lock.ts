import { query } from "@example/db";
query(`SELECT $$ /* deadlock-safe: stored text */ $$ AS note FROM items WHERE id=ANY($1) FOR UPDATE`);
