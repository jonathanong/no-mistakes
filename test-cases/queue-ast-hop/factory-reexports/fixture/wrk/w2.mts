import { createWorker } from "../lib.mts";
import { proc } from "../proc.mts";
import * as procNs from "../proc.mts";

async function localProc(job: { name: string }) {
  switch (job.name) {
    case "vLocalProc":
      return;
  }
}

export const w2 = createWorker("other", proc);
export const wNs = createWorker("other", procNs);
// The factory module has no job names, so this worker stays wildcard.
export const wEmpty = createWorker("unused", createWorker);
// A local function is not an imported processor.
export const wLocal = createWorker("unused-local", localProc);
