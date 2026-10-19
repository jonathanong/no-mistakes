import {
  createBulkEnqueueFunction,
  createEnqueueFunction,
} from "../lib/factory";
import { adminImports, background } from "../queues";

export const enqueuePage = createEnqueueFunction(background, "page");
export const enqueueDead = createEnqueueFunction(adminImports, "dead");
export const enqueueOrphan = createBulkEnqueueFunction(
  background,
  "orphan",
  (row) => row,
);
