const expect = (value: { message: string }) => ({
  toBe(_text: string) {},
});

declare const err: { message: string };

test("local", () => {
  expect(err.message).toBe("x");
  // A nested const named expect shadows the module wrapper and stays silent.
  function helper() {
    const expect = (value: { message: string }) => ({
      toBe(_text: string) {},
    });
    expect(err.message).toBe("x");
  }
  helper();
});
