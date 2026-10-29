import { read } from '@example/db';
// The spelling Error cannot authorize an abrupt proof: this constructor has a SQL effect.
function Error(value: string) {
  read('DROP TABLE hidden');
  return value;
}
export function shadowed(value: string): never {
  throw new Error(value);
}
