// A resolved parse failure is outside the annotation catalog. The recovered
// program has no query facts, so this import cannot yield SQL and does not
// keep the importer.
import { hidden } from "./broken.mts";
export const unused = hidden;
