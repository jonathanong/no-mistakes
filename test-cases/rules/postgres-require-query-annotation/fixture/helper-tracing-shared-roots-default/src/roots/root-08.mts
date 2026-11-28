import { query } from "@bench/db";
import { runAnnotated } from "../shared/helpers.mjs";
import { annotatedTag } from "../shared/constants.mjs";

export function handle08() {
  runAnnotated("items_08", (s) => query(s));
  return query(annotatedTag + "SELECT 8");
}
