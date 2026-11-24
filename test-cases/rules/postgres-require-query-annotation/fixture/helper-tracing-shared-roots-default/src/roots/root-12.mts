import { query } from "@bench/db";
import { runAnnotated } from "../shared/helpers.mjs";
import { annotatedTag } from "../shared/constants.mjs";

export function handle12() {
  runAnnotated("items_12", (s) => query(s));
  return query(annotatedTag + "SELECT 12");
}
