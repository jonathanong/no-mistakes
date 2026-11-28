import { query } from "@bench/db";
import { runAnnotated } from "../shared/helpers.mjs";
import { annotatedTag } from "../shared/constants.mjs";

export function handle06() {
  runAnnotated("items_06", (s) => query(s));
  return query(annotatedTag + "SELECT 6");
}
