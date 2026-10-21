import { query } from "@example/db";
declare const flag: boolean;
query(`SELECT id FROM ${flag ? process.env.SCHEMA : "app"}.users`);
