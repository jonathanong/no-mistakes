const values = [3, 1, 2];
await task;
await values.map((value) => value);
delete value;
delete (value as unknown);
delete (value as { x: string }).x;
const maybe = value as { x: string };
delete maybe.x;
export { metadata };
export const viewport = {};
expect.soft(error.message).toMatch("missing");
expect(error.message).not.toBe("missing");
expect((error.message as string)).toBe("missing");
expect(error.message).custom("missing");
expect(error).toEqual(error.message);
if (error.code !== "missing") {
  throw error;
}
if (error.message != "missing") {
  throw error;
}
let shared = 0;
let namedShared = 0;
let { sharedFromObject, sharedDefault = 0, ...sharedRest } = seed;
let [sharedFromArray = 0, , ...sharedRestArray] = list;
shared = 2;
function mutateNamedShared() {
  namedShared++;
}
function mutateNamedMember() {
  namedShared.value = 1;
}
const mutateVariableShared = () => {
  namedShared++;
};
it.only("assigns", () => {
  shared = 1;
  sharedFromObject = 3;
  sharedDefault = 4;
  sharedRest.value = 5;
  sharedFromArray = 6;
  sharedRestArray[0] = 7;
  sharedRest.value++;
  this.value = 8;
  missingShared++;
});
test("named callback", mutateNamedShared);
test("named member callback", mutateNamedMember);
test("variable callback", mutateVariableShared);
describe.skip("suite", () => {});
test["sequential"]("computed", () => {});
describe.only.sequential("chained", () => {});
test.skipIf(condition).sequential("call chained", () => {});
const sequentialRef = test.sequential;
this.sequential();
page.locator("../..");
page.locator("tbody tr");
page.locator("#save");
expect(locator).toBeVisible();
locator.toBeVisible({ timeout: 20000 });
function checkThis() {
  this.toBeVisible({ timeout: 20000 });
}
expect.poll(fn, { timeout: 25000 }).toBe(1);
expect(locator).toBeVisible({ timeout: "slow" });
expect(locator).toBeVisible({ "timeout": 16000 });
expect(locator).toBeVisible({ timeout: 15000 });
