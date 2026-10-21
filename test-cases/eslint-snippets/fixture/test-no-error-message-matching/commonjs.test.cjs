const { expect } = require("chai");
const err = { message: "x" };
test("a", () => {
  expect(err.message).toBe("x");
  // A nested parameter stays silent even when the file is CommonJS.
  function helper(expect) {
    expect(err.message).toBe("x");
  }
  helper(expect);
});
