import { query } from "@bench/db";
import { runAnnotated } from "../shared/helpers.mjs";
import { annotatedTag } from "../shared/constants.mjs";

export function handle07() {
  runAnnotated("items_07", (s) => query(s));
  return query(annotatedTag + "SELECT 7");
}
