import { query } from "@bench/db";
import { runAnnotated } from "../shared/helpers.mjs";
import { annotatedTag } from "../shared/constants.mjs";

export function handle04() {
  runAnnotated("items_04", (s) => query(s));
  return query(annotatedTag + "SELECT 4");
}
