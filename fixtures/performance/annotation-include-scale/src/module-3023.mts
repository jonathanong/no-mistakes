import { query, createReader, createWriter } from "@bench/db";
import { decorate } from "../shared/provider.mjs";
import type { Replica, Archive } from "@bench/db";

const moduleId = 3023;

export async function run3023(replica: Replica, archive: Archive) {
  const seed = moduleId + 1;
  const reader = createReader();
  const writer = createWriter();
  const branchA = seed > 0 ? seed : 0;
  const branchB = seed % 2 === 0 ? branchA + 2 : branchA + 3;
  const selected = branchB > 2 ? branchB : seed;
  const label = decorate(`module-${moduleId}`);
  if (selected > 0) {
    query("/* module_3023 */ SELECT id FROM records WHERE id = 3023");
    await reader.query("/* reader */ SELECT 1");
    await writer.query("/* writer */ SELECT 2");
    await replica.query("/* replica */ SELECT 3");
    await archive.query("/* archive */ SELECT 4");
  } else {
    await query(`/* module_3023_fallback */ SELECT ${selected}`);
  }
  return label;
}
