import type { TransactionQuery } from './database.mts';
import { MUTATED, ESCAPED, IMPORTER_MUTATED, IMPORTER_ESCAPED, UNKNOWN, RECEIVER, SIBLING_MUTATED, SIBLING_ESCAPED, BARREL_MUTATED } from './control-stages.mts';
IMPORTER_MUTATED[0].run = async () => ({ rowCount: 0 });
declare function escape(value: unknown): void;
escape(IMPORTER_ESCAPED);
export async function controls(query: TransactionQuery) {
  for (const stage of MUTATED) { const result = await stage.run(query); }
  for (const stage of ESCAPED) { const result = await stage.run(query); }
  for (const stage of IMPORTER_MUTATED) { const result = await stage.run(query); }
  for (const stage of IMPORTER_ESCAPED) { const result = await stage.run(query); }
  for (const stage of UNKNOWN) { const result = await stage.run(query); }
  for (const stage of RECEIVER) { const result = await stage.run(query); }
  for (const stage of SIBLING_MUTATED) { const result = await stage.run(query); }
  for (const stage of SIBLING_ESCAPED) { const result = await stage.run(query); }
  for (const stage of BARREL_MUTATED) { const result = await stage.run(query); }
}
