import { query } from '@app/db';

export function runSql(statement: string) {
  return query(statement);
}
