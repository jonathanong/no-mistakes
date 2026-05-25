const myFn = {} as { mockReturnValue(v: unknown): unknown };
myFn.mockReturnValue(42);
test("chain method triggers rule", () => {});
