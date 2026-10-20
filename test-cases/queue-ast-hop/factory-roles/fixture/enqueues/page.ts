import {
  createBulkEnqueueFunction,
  createEnqueueFunction,
} from "../lib/factory";
import { adminImports, background, unattended } from "../queues";

export const enqueuePage = createEnqueueFunction(background, "page");
export const enqueueDead = createEnqueueFunction(adminImports, "dead");
// This queue has no wildcard worker, so its producer must remain unmatched.
export const enqueueUnattended = createEnqueueFunction(unattended, "dead");
export const enqueueOrphan = createBulkEnqueueFunction(
  background,
  "orphan",
  (row) => row,
);
// A string and a member are not local queue bindings, so neither is an edge.
export const enqueueDynamic = createEnqueueFunction("background", "page");
export const enqueueMember = createEnqueueFunction(background.name, "nope");
