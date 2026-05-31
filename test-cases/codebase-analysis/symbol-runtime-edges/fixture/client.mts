import { exec } from "node:child_process";
import * as cp from "node:child_process";

export function runRuntimeEdges() {
  fetch("/api/users/42");
  exec("node worker.mts");
}

export function runMemberRuntimeEdge() {
  cp.exec("node worker.mts");
}

export function formatRuntimeEdges() {
  return "no runtime edges";
}

export function unrelatedRuntimeCall() {
  return Math.max(1, 2);
}

const cache = {
  get() {
    return "cached";
  },
};

const runner = {
  exec() {
    return "not a process spawn";
  },
};

export function unrelatedRuntimeMethods() {
  return [cache.get(), runner.exec()];
}
