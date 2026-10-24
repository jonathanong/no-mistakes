import { query } from "db";
declare const value: string;
query(`SELECT 1 FROM accounts WHERE id = uuid(${value})`);
query(`SELECT 1 FROM accounts WHERE id = CAST(${value} AS uuid)`);
