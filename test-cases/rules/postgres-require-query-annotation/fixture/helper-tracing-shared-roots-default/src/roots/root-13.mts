import { query } from "@bench/db";
import { runAnnotated } from "../shared/helpers.mjs";
import { annotatedTag } from "../shared/constants.mjs";

export function handle13() {
  runAnnotated("items_13", (s) => query(s));
  return query(annotatedTag + "SELECT 13");
}
