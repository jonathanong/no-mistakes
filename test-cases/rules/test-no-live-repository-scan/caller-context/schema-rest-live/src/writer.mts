import { readdirSync } from "node:fs";
export function writeResources({ outputDir, schemaDir }) { return readdirSync(schemaDir); }
