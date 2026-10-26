import { expect as check } from "vitest";
export function assertions(error: Error) {
  const local = check;
  local(error.message).toEqual("missing");
  const expect = check;
  expect(error.message).toMatch("missing");
  local(error.code).toEqual("MISSING");
  // no-mistakes-disable-next-line test-no-error-message-matching -- localized protocol fixture
  local(error.message).toEqual("missing");
}
export function shadowed(expect: Function, check: Function, error: Error) {
  const local = check;
  local(error.message).toEqual("missing");
  expect(error.message).toEqual("missing");
}
export function unrelated(error: Error) {
  const expect = another;
  expect(error.message).toEqual("missing");
}
