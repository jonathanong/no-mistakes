import sql from "sql-template-strings"; const q = sql`SELECT uuid_generate_v4() AS id FROM users`;
