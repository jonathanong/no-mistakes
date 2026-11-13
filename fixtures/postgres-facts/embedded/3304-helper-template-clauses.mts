import { read, sql } from '@example/db';

export function numeric() { return read(`SELECT ${12.5}`); }
export function nestedNumeric() { return read(`SELECT ${`id + ${2}`}`); }
export function literalTrue() { return read(`SELECT ${true ? '1' : missing}`); }
export function literalFalse() { return read(`SELECT ${false ? missing : '2'}`); }
export function correlated(flag: boolean) {
  const column = flag ? 'id' : 'name';
  return read(`SELECT ${column} FROM ${flag ? 'orders' : 'accounts'}`);
}
export function shadowed(flag: boolean) {
  const column = flag ? 'id' : 'name';
  {
    const column = flag ? 'created_at' : 'updated_at';
    read(`SELECT ${column}`);
  }
  return read(`SELECT ${column}`);
}
export function reused(flag: boolean) {
  const column = flag ? 'id' : 'name';
  read(`SELECT ${column}`);
  return read(`SELECT ${column}`);
}
export function unknownArm(flag: boolean) { return read(`SELECT ${flag ? 'id' : missing}`); }
export function nulArm(flag: boolean) { return read(`SELECT ${flag ? 'id' : 'na\0me'}`); }
export function objectArm() { return read(`SELECT ${{ sql: 'id' }}`); }
export function computedArm() { return read(`SELECT ${columns['id']}`); }
export function optionalArm() { return read(`SELECT ${columns?.sql}`); }
export function wrongMember() { return read(`SELECT ${columns.text}`); }
export function shadowedHelper(helper: () => string) { return read(`SELECT ${helper()}`); }
export function optionalHelper() { return read(`SELECT ${leaf?.()}`); }
export function memberHelper() { return read(`SELECT ${helpers.leaf()}`); }
function leaf() { return 'id'; }
export function plainHelper() { return read(`SELECT ${leaf()}`); }
export function unknownHelper() { return read(`SELECT ${missingHelper()}`); }
export function taggedArm() { return read(`SELECT ${sql`id`}`); }
