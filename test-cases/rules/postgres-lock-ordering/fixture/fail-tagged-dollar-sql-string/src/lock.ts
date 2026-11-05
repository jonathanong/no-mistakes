import { query } from "@example/db";
query(`SELECT $note$ /* deadlock-safe: stored text */ $note$ AS note FROM items WHERE id=ANY($1) FOR UPDATE`);
