import { query } from "@data-stores/psql";
export function read(text: string) { return query(text); }
