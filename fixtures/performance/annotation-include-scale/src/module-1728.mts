import { query, createReader, createWriter } from "@bench/db";
import { decorate } from "../shared/provider.mjs";
import type { Replica, Archive } from "@bench/db";

const moduleId = 1728;

export async function run1728(replica: Replica, archive: Archive) {
  const seed = moduleId + 1;
  const reader = createReader();
  const writer = createWriter();
  const branchA = seed > 0 ? seed : 0;
  const branchB = seed % 2 === 0 ? branchA + 2 : branchA + 3;
  const selected = branchB > 2 ? branchB : seed;
  const label = decorate(`module-${moduleId}`);
  if (selected > 0) {
    query("/* module_1728 */ SELECT id FROM records WHERE id = 1728");
    await reader.query("/* reader */ SELECT 1");
    await writer.query("/* writer */ SELECT 2");
    await replica.query("/* replica */ SELECT 3");
    await archive.query("/* archive */ SELECT 4");
  } else {
    await query(`/* module_1728_fallback */ SELECT ${selected}`);
  }
  return label;
}
