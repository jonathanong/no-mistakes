import { enqueueEmail } from "@app/jobs";
import { vi } from "vitest";

// Each unknown or mutable receiver must retain its independent finding.
function conditional(m, condition) {
  let spy;
  if (condition) spy = vi.spyOn(m, "send");
  spy.mockImplementation(() => enqueueEmail());
}
function logical(m, condition) {
  let spy;
  condition && (spy = vi.spyOn(m, "send"));
  spy.mockImplementation(() => enqueueEmail());
}
function afterUse(m) {
  let spy;
  spy.mockImplementation(() => enqueueEmail());
  spy = vi.spyOn(m, "send");
}
function afterTry(m) {
  let spy;
  try { spy = vi.spyOn(m, "send"); } catch {}
  spy.mockImplementation(() => enqueueEmail());
}
function rewritten(m, foreign) {
  let spy;
  spy = vi.spyOn(m, "send");
  spy.mockImplementation(() => enqueueEmail());
  spy = foreign;
}
function alreadyInitialized(m, foreign) {
  let spy = foreign;
  spy = vi.spyOn(m, "send");
  spy.mockImplementation(() => enqueueEmail());
}
function compound(m) {
  let spy;
  spy ||= vi.spyOn(m, "send");
  spy.mockImplementation(() => enqueueEmail());
}
function embedded(m) {
  let spy;
  consume(spy = vi.spyOn(m, "send"));
  spy.mockImplementation(() => enqueueEmail());
}
function unknown(foreign) {
  let spy;
  spy = foreign.spyOn(service, "send");
  spy.mockImplementation(() => enqueueEmail());
}
function shadowed(vi) {
  let spy;
  spy = vi.spyOn(service, "send");
  spy.mockImplementation(() => enqueueEmail());
}
function acrossFunctions(m) {
  let spy;
  function setup() { spy = vi.spyOn(m, "send"); }
  spy.mockImplementation(() => enqueueEmail());
}
function earlierDeclaration(m) {
  spy = vi.spyOn(m, "send");
  let spy;
  spy.mockImplementation(() => enqueueEmail());
}
function initializedAliasWrite(m, replacement) {
  let spy;
  spy = vi.spyOn(m, "send");
  const alias = spy;
  alias.mockImplementation = replacement;
  spy.mockImplementation(() => enqueueEmail());
}
function assignedAliasWrite(m, replacement) {
  let spy, alias;
  spy = vi.spyOn(m, "send");
  alias = spy;
  alias.mockImplementation = replacement;
  spy.mockImplementation(() => enqueueEmail());
}
function propertyWrite(m, replacement) {
  let spy;
  spy = vi.spyOn(m, "send");
  spy.mockImplementation = replacement;
  spy.mockImplementation(() => enqueueEmail());
}
function branched(m, condition) {
  let spy;
  if (condition) { spy = vi.spyOn(m, "send"); }
  spy.mockImplementation(() => enqueueEmail());
}
function closure(m) {
  let spy;
  spy = vi.spyOn(m, "send");
  return () => spy.mockImplementation(() => enqueueEmail());
}
function looseVar(m) {
  var spy;
  spy = vi.spyOn(m, "send");
  spy.mockImplementation(() => enqueueEmail());
}
function update(m) {
  let spy;
  spy++;
  spy.mockImplementation(() => enqueueEmail());
}
function uninitialized() {
  let spy;
  spy.mockImplementation(() => enqueueEmail());
}
function tuple(m) {
  let [spy] = m;
  spy.mockImplementation(() => enqueueEmail());
}
function abrupt(m, stop) {
  let spy;
  spy = vi.spyOn(m, "send");
  if (stop) throw new Error("stop");
  spy.mockImplementation(() => enqueueEmail());
}
