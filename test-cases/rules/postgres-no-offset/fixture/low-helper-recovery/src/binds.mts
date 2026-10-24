import SQL from 'sql-template-strings';
import { read } from '@example/db';
import { nestedBind, invalidBoundRelation, mixedFragment } from './helpers.mjs';
export function binds(value: string, enabled: boolean) {
  read(nestedBind(value)); // CLEAR_BIND
  read(invalidBoundRelation(value)); // UNKNOWN_RELATION
  read(SQL`SELECT ${1}, ${2}, ${3}, ${4}, ${5}, ${6}, ${7}, ${8}, ${9}`.append(SQL`, ${10} AS ten OFFSET 2`)); // OFFSET_TENTH_BIND
  read(SQL`SELECT '${'sql_placeholder_1'}' AS literal`.append(SQL`, ${2}`)); // CLEAR_AUTHORED_MARKER
  read(SQL`SELECT 1`.append(mixedFragment(enabled))); // OFFSET_AND_UNKNOWN
}
