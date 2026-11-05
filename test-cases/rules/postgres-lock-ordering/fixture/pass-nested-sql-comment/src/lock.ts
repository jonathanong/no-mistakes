import { query } from "@example/db";
query(`/* outer /* inner */ deadlock-safe: unique key */ SELECT * FROM items WHERE id=ANY($1) FOR UPDATE`);
