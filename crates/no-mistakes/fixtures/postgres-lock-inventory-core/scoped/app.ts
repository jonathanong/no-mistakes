import { query } from "@example/db";
declare const sql: string;
declare function reserveJob(): void;
void query(sql);
reserveJob();
