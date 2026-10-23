const error = { message: "captured" };
const text = error.message;
const safe = () => {
  expect(text).toBe("captured");
  const isolated = () => {
    eval("var text = 'ordinary'");
  };
  isolated();
};
const check = () => {
  // In a non-strict function, direct eval can introduce a nearer ordinary binding.
  eval("var text = 'ordinary'");
  expect(text).toBe("ordinary");
};
const nestedBlock = () => {
  {
    eval("var text = 'ordinary'");
  }
  expect(text).toBe("ordinary");
};
safe();
check();
nestedBlock();
