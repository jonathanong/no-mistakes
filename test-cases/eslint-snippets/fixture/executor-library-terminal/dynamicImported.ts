import { type SessionExecutor } from "@example/db";
import * as adapters from "./dynamicResolvers";

export async function importedDynamic(query: SessionExecutor, id: string, column: string) {
  await adapters.appendedAdapter(id, column, { query });
  await adapters.suffixedAdapter(id, column, { query });
  await adapters.untrustedAdapter(id, { query });
  await adapters.mixedAdapter(id, column, { query });
}
