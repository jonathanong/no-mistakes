import { beginTransaction } from "@example/db";
declare const dynamicSql: string;
declare function opaque(value: unknown): void;

export async function factoryReturnData(
  id: string,
  dependencies?: { begin?: typeof beginTransaction },
) {
  const begin = dependencies?.begin ?? beginTransaction; // NON_SQL
  await using tx = await begin(); // NON_SQL
  const { rows } = await tx(dynamicSql); // ADVISE_FACTORY_QUERY
  await tx.commit(); // NON_SQL

  // Executing SQL returns data, not another producer or executor carrier.
  const row = rows[0] as { id?: string } | undefined; // NON_SQL
  if (row?.id === id) return; // NON_SQL

  await tx(dynamicSql); // ADVISE_SECOND_FACTORY_QUERY
  opaque(tx); // ADVISE_FACTORY_EXECUTOR_ESCAPE
}
