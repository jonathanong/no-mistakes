import { type SessionExecutor } from "@example/db";
import * as adapters from "./dynamicResolvers";

export async function importedProven(query: SessionExecutor, id: string) {
  await adapters.provenAdapter(id, { query });
}
