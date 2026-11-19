import { type SessionExecutor } from "@example/db";
import * as helpers from "./resolvers";

export async function importedTerminal(query: SessionExecutor, id: string) {
  await helpers.passOptions(id, { query });
  await helpers.spreadOverride(id, { query });
  await helpers.optionalNoDefault(id, { query });
}

export async function importedClosures(query: SessionExecutor, id: string) {
  await helpers.fromExpression(id, { query });
  await helpers.fromBlock(id, { query });
  await helpers.fromFunction(id, { query });
}
