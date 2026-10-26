function expect(value: { message: string }) {
  return { toBe(_text: string) {} };
}

declare const err: { message: string };

test("function", () => {
  expect(err.message).toBe("x");
});
