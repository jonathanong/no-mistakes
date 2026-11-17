import { query } from '@example/db';
import { pick } from '../parameter-cast-source';

type Kind = 'orders' | 'refunds';

export function throughReturn(kind: Kind) {
  kind = pick(kind) as string as Kind;
  return query(`SELECT 1 FROM ${kind}_accounts`);
}

export function throughObject(kind: Kind) {
  return query(`SELECT 1 FROM ${({ table: kind as string } as const).table}_accounts`);
}

export function throughArray(kind: Kind) {
  return query(`SELECT 1 FROM ${([kind as string] as const)[0]}_accounts`);
}
