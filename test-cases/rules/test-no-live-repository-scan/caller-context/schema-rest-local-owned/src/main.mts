import { writeResources } from "./writer.mts";
const DEFAULT_SCHEMA_DIR = process.cwd();
export function main(options) { const { outputDir, ...paths } = options; return writeResources({ outputDir, schemaDir: paths.schemaDir ?? DEFAULT_SCHEMA_DIR }); }
