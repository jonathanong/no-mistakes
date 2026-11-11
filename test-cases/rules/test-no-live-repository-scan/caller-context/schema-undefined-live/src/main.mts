import { writeResources } from "./writer.mts";
const DEFAULT_SCHEMA_DIR = process.cwd();
export function main(paths) {
  const outputDir = paths.outputDir;
  return writeResources({ outputDir, schemaDir: paths.schemaDir ?? DEFAULT_SCHEMA_DIR });
}
export function production() { return main({ outputDir: "out", schemaDir: DEFAULT_SCHEMA_DIR }); }
