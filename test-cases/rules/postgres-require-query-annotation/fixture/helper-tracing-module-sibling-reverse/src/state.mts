import sql from 'sql-template-strings';
import { write } from '@app/db';
const statement = sql`/* module annotation */ SELECT 1`;
export function mutate() {
  const changed = unknownMutation(statement);
}
export function send() {
  write(statement); // sibling send preserves original annotation
}
