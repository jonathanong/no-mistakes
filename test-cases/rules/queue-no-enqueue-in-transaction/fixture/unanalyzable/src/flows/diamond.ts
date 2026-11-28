import { label, missingTx } from "../diamond-barrel";
import type { Label } from "../diamond-barrel";
import { transaction } from "@sample/database";

await missingTx("write", async () => label.length);

export function readLabel(value: Label) {
  return transaction("write", async () => value.toString());
}
