import { query } from '@example/db';

export function resetTables() {
  return query('DROP TABLE helper_test');
}
