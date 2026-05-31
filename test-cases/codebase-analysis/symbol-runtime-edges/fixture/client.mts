import { exec } from "node:child_process";

export function runRuntimeEdges() {
  fetch("/api/users/42");
  exec("node worker.mts");
}

