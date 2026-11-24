import { query } from "@bench/db";
import { runAnnotated } from "../shared/helpers.mjs";
import { annotatedTag } from "../shared/constants.mjs";

export function handle10() {
  runAnnotated("items_10", (s) => query(s));
  return query(annotatedTag + "SELECT 10");
}
