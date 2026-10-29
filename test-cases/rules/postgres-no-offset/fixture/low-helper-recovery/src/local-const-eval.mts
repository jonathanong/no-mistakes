import SQL from 'sql-template-strings';
import { read } from '@example/db';

export function dynamicMutation() {
  const map = { sql: SQL`SELECT 1` };
  // Direct eval can mutate lexical bindings without a static assignment node.
  eval("map.sql = 'DROP TABLE hidden'");
  read(map.sql); // RESIDUAL: dynamic lexical mutation invalidates const recovery.
}
