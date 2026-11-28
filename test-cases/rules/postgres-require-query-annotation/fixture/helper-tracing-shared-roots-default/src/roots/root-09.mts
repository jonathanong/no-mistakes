import { query } from "@bench/db";
import { runAnnotated } from "../shared/helpers.mjs";
import { annotatedTag } from "../shared/constants.mjs";

export function handle09() {
  runAnnotated("items_09", (s) => query(s));
  return query(annotatedTag + "SELECT 9");
}
