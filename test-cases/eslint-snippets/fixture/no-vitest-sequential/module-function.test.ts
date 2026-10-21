function test(_name: string, _fn: () => void) {}

test.sequential("module function", () => {});

// A parameter named test is not the module function.
function helper(test: { sequential(name: string, fn: () => void): void }) {
  test.sequential("nested shadow", () => {});
}
