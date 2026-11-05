import { query } from "@example/db";
query(`SELECT $not_a_tag FROM items WHERE id=ANY($1) FOR UPDATE`);
