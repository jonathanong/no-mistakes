import { query } from "@bench/db";
import { runAnnotated } from "../shared/helpers.mjs";
import { annotatedTag } from "../shared/constants.mjs";

export function handle11() {
  runAnnotated("items_11", (s) => query(s));
  return query(annotatedTag + "SELECT 11");
}
