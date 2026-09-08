import { query } from "@data-stores/psql";

declare const tableName: string;

query(`SELECT * FROM ${tableName}`);
