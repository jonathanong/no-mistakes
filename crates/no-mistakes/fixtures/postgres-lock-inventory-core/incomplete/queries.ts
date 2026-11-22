import { query } from "@example/db";

declare function reserveJob(id: number): void;
declare const sql: string;
reserveJob(1);
void query(sql);
