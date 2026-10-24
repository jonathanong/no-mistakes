import SQL from 'sql-template-strings';

export function ambiguousFragment() {
  return SQL` WHERE two = true`;
}
