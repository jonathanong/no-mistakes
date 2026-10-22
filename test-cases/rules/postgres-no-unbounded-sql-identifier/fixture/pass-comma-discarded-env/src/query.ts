import { query } from "@example/db";
// Only the final comma operand supplies the identifier.
query(`SELECT id FROM ${(process.env.SCHEMA, "public")}.users`);
