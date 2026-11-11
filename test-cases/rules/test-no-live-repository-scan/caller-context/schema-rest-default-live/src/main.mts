import { writeResources } from "./writer.mts";
const DEFAULT_SCHEMA_DIR = process.cwd();
export function main({ outputDir, ...paths } = { schemaDir: process.cwd() }) { return writeResources({ outputDir, schemaDir: paths.schemaDir ?? DEFAULT_SCHEMA_DIR }); }
