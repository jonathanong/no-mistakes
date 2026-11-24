import { query } from "@bench/db";
import { runAnnotated } from "../shared/helpers.mjs";
import { annotatedTag } from "../shared/constants.mjs";

export function handle14() {
  runAnnotated("items_14", (s) => query(s));
  return query(annotatedTag + "SELECT 14");
}
