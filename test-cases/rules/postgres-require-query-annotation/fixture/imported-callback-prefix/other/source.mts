import { sql } from './sql.mjs';
export function foreignStatement() {
  return sql`/* same spelling, different provider */ SELECT id FROM widgets`;
}
