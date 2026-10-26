// Name-based test/it reporting predates binding-aware runner alias support.
export function parameter(test) {
  test("case", () => error.message === "expected");
}
export function nested() {
  const test = unknown;
  test("case", () => error.message === "expected");
}
export function includes(it) {
  it("case", () => error.message.includes("expected"));
}
export function computed(test, key) {
  test[key]("opaque", () => error.message === "expected");
}
export function renamed(check) {
  check("unproven", () => error.message === "expected");
}
export function generated(test) {
  test.each([1])("case", () => error.message === "expected");
}
const test = customRunner;
const run = test;
run("module alias", () => error.message === "expected");
// Arbitrary IIFEs do not establish a test callback boundary.
(() => error.message === "outside")();
export function unprovenAlias() {
  const test = unknown;
  const renamed = test;
  renamed("opaque", () => error.message === "expected");
}
