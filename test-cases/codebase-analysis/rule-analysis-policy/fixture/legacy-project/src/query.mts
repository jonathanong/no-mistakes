import { query } from '@example/db';
import { tables } from './tables.mjs';
declare const process: { env: { SCHEMA: string } };
query(`SELECT * FROM ${tables.users}`);
query(`SELECT * FROM ${process.env.SCHEMA}.users`);
