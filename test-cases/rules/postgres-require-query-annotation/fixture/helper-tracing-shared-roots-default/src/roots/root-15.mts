import { query } from "@bench/db";
import { runAnnotated, runRaw } from "../shared/helpers.mjs";
import { annotatedTag } from "../shared/constants.mjs";
import { runImpure } from "../shared/impure.mjs";

export function handle15() {
  runAnnotated("items_15", (s) => query(s));
  runRaw("SELECT id FROM missing_root_15", (s) => query(s));
  query(annotatedTag + "SELECT 15");
  return runImpure("/* impure_call */ SELECT 15");
}
