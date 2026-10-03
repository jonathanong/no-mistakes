import { query } from "@example/db";
export function read(text: string) { return query(text); }
