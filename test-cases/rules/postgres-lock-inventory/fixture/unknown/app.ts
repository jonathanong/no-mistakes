import { query } from "@example/db";
export function reserve(sql: string) { return query(sql); }
