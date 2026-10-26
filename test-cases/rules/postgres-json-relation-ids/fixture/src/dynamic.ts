import { query } from "@example/db";
export const dynamic = (statement: string) => query(statement);
export const interpolatedTarget = (table: string) => query(`INSERT INTO ${table} (details) VALUES ('{"accountId": "a1"}'::jsonb)`);
const prose = `INSERT INTO orders (details) VALUES ('{"accountId": "a1"}'::jsonb)`;
export const unexecuted = () => prose;
