import { mkdtempSync, readdirSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

function withTempDirectory(run: (root: string) => void) {
  const root = mkdtempSync(join(tmpdir(), "prefix-"));
  try { run(root); } finally { rmSync(root, { force: true, recursive: true }); }
}
withTempDirectory((root) => {
  writeArtifact(root);
  symlinkSync(join(root, "index.html"), join(root, "leak"));
  expect(() => readdirSync(root, "nested-direct")).toThrow("Unsafe");
  rmSync(join(root, "leak"));
  for (const reserved of ["a", "b"]) {
    const path = join(root, reserved);
    writeFileSync(path, "unsafe");
    expect(() => readdirSync(root, "nested-loop")).toThrow("x");
    rmSync(path);
  }
});
withTempDirectory((root) => {
  { { expect(() => readdirSync(root, "nested-block")).toThrow("x"); } }
});
withTempDirectory((root) => {
  expect(() => {
    const root = process.cwd();
    readdirSync(root, "shadowed");
  }).toThrow("x");
});
withTempDirectory((root) => {
  expect(() => {
    root = process.cwd();
    readdirSync(root, "reassigned");
  }).toThrow("x");
});
withTempDirectory((root) => {
  for (const reserved of ["a", "b"]) {
    return Promise.resolve().then(() => readdirSync(root, "loop-returned-promise"));
  }
});
function withUnownedRoot(run: (root: string) => void) {
  const root = process.cwd();
  try { run(root); } finally { rmSync(root, { recursive: true }); }
}
withUnownedRoot((root) => {
  for (const reserved of ["a", "b"]) {
    expect(() => readdirSync(root, "non-owned")).toThrow("x");
  }
});
