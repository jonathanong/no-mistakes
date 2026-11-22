import { run0007 } from "../src/module-0007.mjs";
import { createReader, createWriter } from "@bench/db";
import type { Replica, Archive } from "@bench/db";

export function invokeFromExcludedModule(replica: Replica, archive: Archive) {
  return run0007(replica, archive);
}

void createReader;
void createWriter;
