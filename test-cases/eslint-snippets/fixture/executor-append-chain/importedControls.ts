import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";
import { mutatedHelper, parameterHelper, parameterizedHelper } from "./helperControls";

export async function importedControls(query: SessionExecutor, id: string) {
  await mutatedHelper(query, id);
  await parameterHelper(query, id, sql`id`);
  await parameterizedHelper(query, id);
}
