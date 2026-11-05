import { query } from "@example/db";
query(`SELECT $$stored text$$ AS note /* deadlock-safe: unique key */ FROM items WHERE id=ANY($1) FOR UPDATE`);
