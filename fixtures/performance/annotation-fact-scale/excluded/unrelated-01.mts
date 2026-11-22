import { query } from "@bench/db";
export function unrelated01() { return query("SELECT 1"); }
