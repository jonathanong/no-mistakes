import { mkdtempSync, mkdtempDisposableSync, readdirSync, rmSync, unlinkSync, rmSync as writtenCleanup } from "node:fs";
import { rm as asyncRm } from "node:fs/promises";
import { rmSync as wrongModuleCleanup } from "other-filesystem";
import type { rmSync as typeCleanup } from "node:fs";
import { type rmSync as inlineTypeCleanup } from "node:fs";
import * as fs from "node:fs";
export const outsideRoot = mkdtempSync("callback-");
writtenCleanup = () => {};

const withRoot0 = function (run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { run(root); } finally { rmSync(root, { recursive: true }); }
};
withRoot0((root) => { readdirSync(root, "function-expression"); });

function withRoot1(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  type Directory = typeof root;
  try { run(root); } finally { rmSync(root, { recursive: true }); }
}
withRoot1((root) => { readdirSync(root, "root-type-reference"); });

function withRoot2(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  type Runner = typeof run;
  try { run(root); } finally { rmSync(root, { recursive: true }); }
}
withRoot2((root) => { readdirSync(root, "callback-type-reference"); });

function withRoot3(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { run(root); } finally { rmSync(root, { force: true, recursive: true }); }
}
withRoot3((root) => { readdirSync(root, "cleanup-extra-option"); });

function withRoot4(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { run(root); } finally { rmSync(root, { recursive: true }); }
}
withRoot4((root) => { readdirSync(root, "spread-supply"); }, ...[]);

function withRoot5(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { run(root); } finally { rmSync(root, { recursive: true }); }
}
withRoot5(function* (root) { readdirSync(root, "generator-callback"); });

const withRoot6 = 42;
withRoot6((root) => { readdirSync(root, "nonfunction-helper"); });

function* withRoot7(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { run(root); } finally { rmSync(root, { recursive: true }); }
}
withRoot7((root) => { readdirSync(root, "generator-helper"); });

function withRoot8(run: (root: string) => void = () => {}) {
  const root = mkdtempSync("callback-");
  try { run(root); } finally { rmSync(root, { recursive: true }); }
}
withRoot8((root) => { readdirSync(root, "default-callback"); });

function withRoot9({ run }: { run: (root: string) => void }) {
  const root = mkdtempSync("callback-");
  try { run(root); } finally { rmSync(root, { recursive: true }); }
}
withRoot9((root) => { readdirSync(root, "destructured-callback"); });

function withRoot10(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  run = () => {};
  try { run(root); } finally { rmSync(root, { recursive: true }); }
}
withRoot10((root) => { readdirSync(root, "callback-write"); });

function withRoot11(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { (() => run(root))(); } finally { rmSync(root, { recursive: true }); }
}
withRoot11((root) => { readdirSync(root, "nested-invocation"); });

function withRoot12(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { run(mkdtempDisposableSync("callback-").path); } finally { rmSync(root, { recursive: true }); }
}
withRoot12((root) => { readdirSync(root, "root-member-call"); });

function withRoot13(run: (root: string) => void, root: string) {
  try { run(root); } finally { rmSync(root, { recursive: true }); }
}
withRoot13((root) => { readdirSync(root, "root-parameter"); });

function withRoot14(run: (root: string) => void) {
  const { root } = { root: mkdtempSync("callback-") };
  try { run(root); } finally { rmSync(root, { recursive: true }); }
}
withRoot14((root) => { readdirSync(root, "root-destructure"); });

function withRoot15(run: (root: string) => void) {

  try { run(outsideRoot); } finally { rmSync(outsideRoot, { recursive: true }); }
}
withRoot15((root) => { readdirSync(root, "root-global"); });

function withRoot16(run: (root: string) => void) {
  { const root = mkdtempSync("callback-");
  try { run(root); } finally { rmSync(root, { recursive: true }); }
  }
}
withRoot16((root) => { readdirSync(root, "root-block"); });

function withRoot17(run: (root: string) => void) {
  try { run(root); } finally { rmSync(root, { recursive: true }); }
  const root = mkdtempSync("callback-");
}
withRoot17((root) => { readdirSync(root, "root-late"); });

function withRoot18(run: (root: string) => void) {
  const root = "fixtures";
  try { run(root); } finally { rmSync(root, { recursive: true }); }
}
withRoot18((root) => { readdirSync(root, "root-literal"); });

function withRoot19(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  root = "other";
  try { run(root); } finally { rmSync(root, { recursive: true }); }
}
withRoot19((root) => { readdirSync(root, "root-write"); });

function withRoot20(run: (root: string) => void) {
  using root = mkdtempDisposableSync("callback-");
  try { run(root.path); root.remove(); } finally {  }
}
withRoot20((root) => { readdirSync(root, "resource-member-other"); });

function withRoot21(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { run(root); } catch {}
}
withRoot21((root) => { readdirSync(root, "cleanup-no-finally"); });

function withRoot22(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { run(root); } finally { 42; }
}
withRoot22((root) => { readdirSync(root, "cleanup-noncall"); });

function withRoot23(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { run(root); } finally { const beforeCleanup = 1; rmSync(root, { recursive: true }); }
}
withRoot23((root) => { readdirSync(root, "cleanup-first-expression"); });

function withRoot24(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { run(root); } finally { fs.rmSync(root, { recursive: true }); }
}
withRoot24((root) => { readdirSync(root, "cleanup-namespace"); });

function withRoot25(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { run(root); } finally { unboundCleanup(root, { recursive: true }); }
}
withRoot25((root) => { readdirSync(root, "cleanup-global"); });

function withRoot26(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  const cleanup = () => {};
  try { run(root); } finally { cleanup(root, { recursive: true }); }
}
withRoot26((root) => { readdirSync(root, "cleanup-local"); });

function withRoot27(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { run(root); } finally { writtenCleanup(root, { recursive: true }); }
}
withRoot27((root) => { readdirSync(root, "cleanup-written"); });

function withRoot28(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { run(root); } finally { typeCleanup(root, { recursive: true }); }
}
withRoot28((root) => { readdirSync(root, "cleanup-type-import"); });

function withRoot29(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { run(root); } finally { inlineTypeCleanup(root, { recursive: true }); }
}
withRoot29((root) => { readdirSync(root, "cleanup-inline-type-import"); });

function withRoot30(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { run(root); } finally { unlinkSync(root, { recursive: true }); }
}
withRoot30((root) => { readdirSync(root, "cleanup-wrong-export"); });

function withRoot31(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { run(root); } finally { wrongModuleCleanup(root, { recursive: true }); }
}
withRoot31((root) => { readdirSync(root, "cleanup-wrong-module"); });

function withRoot32(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { run(root); } finally { rmSync(); }
}
withRoot32((root) => { readdirSync(root, "cleanup-no-argument"); });

function withRoot33(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { run(root); } finally { rmSync(root.path, { recursive: true }); }
}
withRoot33((root) => { readdirSync(root, "cleanup-member-argument"); });

function withRoot34(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { run(root); } finally { rmSync(root); }
}
withRoot34((root) => { readdirSync(root, "cleanup-no-options"); });

function withRoot35(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { run(root); } finally { rmSync(root, options); }
}
withRoot35((root) => { readdirSync(root, "cleanup-options-variable"); });

function withRoot36(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { run(root); } finally { rmSync(root, { ...options, recursive: true }); }
}
withRoot36((root) => { readdirSync(root, "cleanup-spread-options"); });

function withRoot37(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { run(root); } finally { rmSync(root, { ["recursive"]: true }); }
}
withRoot37((root) => { readdirSync(root, "cleanup-computed-options"); });

function withRoot38(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { run(root); } finally { asyncRm(root, { recursive: true }); }
}
withRoot38((root) => { readdirSync(root, "cleanup-async-unawaited"); });


function withOtherRoot(run: (root: string) => void) {
  const root = mkdtempSync("callback-");
  try { run(root); } finally { rmSync(outsideRoot, { recursive: true }); }
}
withOtherRoot((root) => { readdirSync(root, "cleanup-other-root"); });

function withResourceProperty(run: (root: string) => void) {
  using resource = mkdtempDisposableSync("callback-");
  resource.other;
  run(resource.path);
}
withResourceProperty((root) => { readdirSync(root, "resource-other-read"); });
