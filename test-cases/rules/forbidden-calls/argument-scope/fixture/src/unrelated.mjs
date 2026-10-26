import { execFile } from "node:child_process";
// A target-only application must not initialize argument collection here.
export function unrelated() {
  const options = { values: ["one", "two", "three"], nested: { enabled: true } };
  execFile("live", options.values);
  setTimeout(() => options.nested.enabled, 1);
}
unrelated();
