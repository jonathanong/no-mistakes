import sql from "sql-template-strings"; import { read } from '@example/db';
const result = await read(sql`SELECT id FROM users WHERE id = $1`, [id]);
