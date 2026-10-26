import { query } from '@example/db';

export function runDynamic(statement: string) {
  return query(statement);
}
