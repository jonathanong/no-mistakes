import { query } from "@bench/db";
import { runAnnotated } from "../shared/helpers.mjs";
import { annotatedTag } from "../shared/constants.mjs";

export function handle03() {
  runAnnotated("items_03", (s) => query(s));
  return query(annotatedTag + "SELECT 3");
}
