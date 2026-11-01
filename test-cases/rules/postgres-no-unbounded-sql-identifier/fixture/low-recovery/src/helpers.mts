import sql from 'sql-template-strings';

export function active() {
  return sql` AND active = true`;
}

export function dynamicName(table: string) {
  return `SELECT * FROM ${table}`;
}
