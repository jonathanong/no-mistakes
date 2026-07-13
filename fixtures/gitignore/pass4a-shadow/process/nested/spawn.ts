import { spawn } from "node:child_process";

export function startWorker() {
  return spawn("worker.ts");
}
