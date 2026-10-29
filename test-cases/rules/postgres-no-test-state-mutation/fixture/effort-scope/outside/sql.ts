import { query } from '@example/db';

// Deliberately present in the inventory but outside configured helper roots.
export function outsideSql() {
  return 'DROP TABLE app.outside_helper';
}

export function outsideEffect() {
  return query('SELECT 1');
}
