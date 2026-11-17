import { write } from "../test-helpers/write.mts";

export async function exercise(dynamicArg: string, kind: string, provider: string) {
  await write(dynamicArg, kind, provider);
}
