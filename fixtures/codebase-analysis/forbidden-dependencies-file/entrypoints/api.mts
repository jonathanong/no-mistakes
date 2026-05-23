import { run } from "./worker.mts";

export function handleRequest() {
  return run();
}
