import { query } from "@bench/db";
import { runAnnotated } from "../shared/helpers.mjs";
import { annotatedTag } from "../shared/constants.mjs";

export function handle05() {
  runAnnotated("items_05", (s) => query(s));
  return query(annotatedTag + "SELECT 5");
}
