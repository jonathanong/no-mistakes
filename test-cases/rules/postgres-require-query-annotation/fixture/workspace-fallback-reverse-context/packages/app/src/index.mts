import { query } from '@app/db';
// An excluded caller supplies the SQL observed at this selected executor span.
export function run(sql: string) { return query(sql); }
