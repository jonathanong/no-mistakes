import { type SessionExecutor } from "@example/db";
import { exportedHelper, localConstHelper } from "./helpers";

export async function importedHelpers(query: SessionExecutor, id: string) {
  await exportedHelper(query, id);
  await localConstHelper(query, id);
}
