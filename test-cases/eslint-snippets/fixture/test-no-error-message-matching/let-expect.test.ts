let expect: (value: { message: string }) => { toBe(text: string): void };

declare const err: { message: string };

test("let", () => {
  expect(err.message).toBe("x");
});
