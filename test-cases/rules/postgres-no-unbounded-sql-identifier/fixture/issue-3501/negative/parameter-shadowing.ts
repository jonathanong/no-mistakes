import { query } from '@example/db';

type Kind = 'orders' | 'refunds';

export function shadowedParameter(kind: Kind, raw: string) {
  const fixed = query(`SELECT 1 FROM ${kind}_accounts`);
  function run(kind: string) {
    return query(`SELECT 1 FROM ${kind}_accounts`);
  }
  return [fixed, run(raw)];
}
