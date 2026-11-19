import { type SessionExecutor } from "@example/db";
import * as resolvers from "./resolvers";

export async function importedForward(query: SessionExecutor, id: string) {
  return resolvers.rejectUnresolvable(id, { query });
}
