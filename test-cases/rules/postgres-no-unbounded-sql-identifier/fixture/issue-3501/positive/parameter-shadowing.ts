import { query } from '@example/db';

type Kind = 'orders' | 'refunds';

export function shadowedAssignment(kind: Kind, replacement: string) {
  function replace(kind: string) {
    kind = replacement;
    return kind;
  }
  replace(kind);
  return query(`SELECT 1 FROM ${kind}_accounts`);
}
