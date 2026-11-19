import { type SessionExecutor } from "@example/db";
import * as helpers from "./helpers";

export async function importedBang(query: SessionExecutor, id: string) {
  await helpers.bangReceiver(id, { query });
}

export async function importedClosure(query: SessionExecutor, id: string) {
  await helpers.fromClosure(id, { query });
}
