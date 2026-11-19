import { type SessionExecutor } from "@example/db";
import * as helpers from "./unsafeResolvers";

export async function importedReports(query: SessionExecutor, id: string) {
  await helpers.dynamicStatement(id, { query });
  await helpers.undeclaredStatement({ query });
  await helpers.manyStatements(id, { query });
  await helpers.transformsStatement(id, { query });
  await helpers.asyncClosure(id, { query });
  await helpers.generatorClosure(id, { query });
  await helpers.twoParameters(id, { query });
  await helpers.notAnArgument(id, { query });
}
