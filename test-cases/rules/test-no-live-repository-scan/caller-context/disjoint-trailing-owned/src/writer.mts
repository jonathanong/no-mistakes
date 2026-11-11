import { readdirSync } from "node:fs";
export function writeResources({ outputRoot, consumerRoot, check, schemaDir }) {
  return readdirSync(outputRoot);
}
