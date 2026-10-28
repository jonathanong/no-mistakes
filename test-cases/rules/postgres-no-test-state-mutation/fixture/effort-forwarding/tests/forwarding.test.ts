import { beginTransaction } from '../db/index';

declare function unknownSql(): string;
declare function opaque(value: unknown): void;
function mutate(executor: (sql: string) => unknown) {
  return executor('DROP TABLE app.forwarded_mutation');
}

export async function run() {
  const tx = await beginTransaction();
  opaque(tx); // first executor forwarding
  opaque(tx); // second executor forwarding
  mutate(tx); // helper executes a known mutation
  await tx(unknownSql()); // unresolved selected sink
}
