import { query } from "@bench/db";
import { runAnnotated } from "../shared/helpers.mjs";
import { annotatedTag } from "../shared/constants.mjs";

export function handle02() {
  runAnnotated("items_02", (s) => query(s));
  return query(annotatedTag + "SELECT 2");
}
