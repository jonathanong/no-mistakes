import { query } from '@example/db';
type Kind = OtherKind;
type OtherKind = Kind;
export function cyclic(kind: Kind) {
  return query(`SELECT 1 FROM ${kind}_accounts`);
}
