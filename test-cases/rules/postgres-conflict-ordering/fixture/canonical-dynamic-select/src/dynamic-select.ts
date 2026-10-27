import { query } from "@example/db";

declare const tableName: string;

query(`SELECT * FROM ${tableName}`);
