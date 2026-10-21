import { query } from "@example/db";
declare const key: string;
query(`SELECT id FROM ${process.env[key]}.users`);
